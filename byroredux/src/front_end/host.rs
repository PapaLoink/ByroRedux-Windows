//! Native host glue. Menu/world lifetimes remain separate until a validated load.
use super::*;
use byroredux_debug_ui::{egui, panels::PanelOutputs};
use winit::{event::WindowEvent,event_loop::ActiveEventLoop};

fn show_options_status(ctx:&egui::Context,status:Option<&str>) {
    let Some(status)=status.filter(|s|!s.is_empty()) else {return;};
    // An empty label lets an Area remember a near-zero width. Reusing that
    // width for a longer status wraps it into a column over the main menu.
    let width=(ctx.content_rect().width()-32.0).clamp(1.0,720.0);
    egui::Area::new(egui::Id::new("front_end_options_status"))
        .anchor(egui::Align2::CENTER_BOTTOM,egui::vec2(0.0,-12.0))
        .interactable(false).show(ctx,|ui|{
            ui.set_width(width);
            ui.label(status);
        });
}

impl crate::App {
    pub(crate) fn start_front_end(&mut self,event_loop:&ActiveEventLoop)->bool {
        let args=crate::cli_args::effective_args();
        if !requested(&args) {return false;}
        let result=(||->Result<FrontEnd>{
            let (esm,data)=crate::cli_args::esm_discovery_root(&args).map_err(anyhow::Error::msg)?;
            if !std::path::Path::new(&esm).file_name().is_some_and(|n|n.eq_ignore_ascii_case("Oblivion.esm")) {
                bail!("--oblivion-main-menu requires original Oblivion.esm");
            }
            let front=FrontEnd::new(&data,&self.world)?;
            self.renderer.as_mut().unwrap().register_ui_quad()?;
            Ok(front)
        })();
        match result {Ok(front)=>self.front_end=Some(front),Err(error)=>{log::error!("Original front end unavailable: {error:#}");event_loop.exit();}}
        true
    }
    pub(crate) fn front_end_window_event(&mut self,event_loop:&ActiveEventLoop,event:&WindowEvent)->bool {
        if self.front_end.is_none(){return false;}
        if matches!(event,WindowEvent::CloseRequested) {self.shutdown(event_loop);return true;}
        let win=self.window.as_ref().unwrap();
        let native=self.front_end.as_ref().is_some_and(|f|f.show_saves||f.show_new||f.notice.is_some());
        let consumed=self.debug_ui.as_mut().is_some_and(|ui|ui.on_window_event(win,event).consumed);
        if let WindowEvent::Resized(size)=event {
            let ctx=self.renderer.as_mut().unwrap();
            let output=ctx.frame_extents.output;
            if crate::app_events::should_recreate_for_window_resize(
                [output.width,output.height],[size.width,size.height],
            ) {
                if let Err(error)=ctx.recreate_swapchain([size.width,size.height]) {
                    log::error!("Front-end resize failed: {error:#}");event_loop.exit();
                }
            }
        }
        if !consumed || !native {
            let front=self.front_end.as_mut().unwrap();
            let (w,h)=front.menu.frame_size();
            let mut audio=self.world.resource_mut::<AudioWorld>();
            let mut error=None;
            crate::ui_input::dispatch_window_event(event,&mut self.ui_input_state,win.inner_size(),(w,h),|input|{
                if let Err(e)=front.input(input,&mut audio){error=Some(e);}
            });
            if let Some(error)=error {log::error!("Front-end input failed: {error:#}");event_loop.exit();}
        }
        true
    }
    fn close_front_end(&mut self,front:&mut FrontEnd) {
        self.world.resource_mut::<AudioWorld>().stop_music(0.0);
        self.world.resource_mut::<AudioWorld>().stop_presentation();
        front.movie.take();
        if let Some(texture)=front.texture.take(){let ctx=self.renderer.as_mut().unwrap();ctx.texture_registry.drop_texture(&ctx.device,texture);}
        // Menu clicks and key presses must not become the first gameplay action.
        self.ui_input_state=Default::default();
    }
    pub(crate) fn step_front_end(&mut self,event_loop:&ActiveEventLoop)->bool {
        let Some(mut front)=self.front_end.take() else{return false;};
        #[cfg(feature="debug-server")]
        self.scheduler.run_exclusive_named(&self.world,0.0,byroredux_debug_server::DRAIN_SYSTEM_NAME);
        let result=(||->Result<Action>{
            let ctx=self.renderer.as_mut().unwrap();
            front.tick(&self.world,ctx)?;
            let mut outputs=PanelOutputs::default();
            if let (Some(ui),Some(win))=(self.debug_ui.as_mut(),self.window.as_ref()) {
                ui.run_front_end(win,|egui_ctx|{
                    if front.phase==Phase::Menu {
                        show_options_status(egui_ctx,front.settings.as_ref().map(|settings|settings.status.as_str()));
                    }
                    if egui_ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        front.show_saves=false;front.show_new=false;front.notice=None;
                    }
                    if front.show_new {
                        egui::Window::new("New game").collapsible(false).show(egui_ctx,|ui|{
                            ui.label("Start a fresh session in the original Imperial Prison?");
                            ui.label("Partial support: character creation and the complete tutorial sequence are not yet implemented.");
                            if ui.button("Start").clicked(){front.action=Action::New;}
                            if ui.button("Back").clicked(){front.show_new=false;}
                        });
                    }
                    if front.show_saves {
                        egui::Window::new("Engine saves — original Bethesda saves unsupported").collapsible(false).show(egui_ctx,|ui|{
                            if front.saves.is_empty() {
                                ui.label("No compatible Oblivion engine saves found.");
                                ui.label("Use F5 during gameplay to create an engine save. Original Bethesda saves cannot be loaded yet.");
                            }
                            if ui.button("Refresh").clicked(){front.saves=saves::slots(&self.world);}
                            for save in &front.saves {
                                if ui.button(&save.label).clicked(){front.action=Action::Load(save.number);}
                            }
                            if ui.button("Back").clicked(){front.show_saves=false;}
                        });
                    }
                    if let Some(notice)=front.notice.clone() {
                        egui::Window::new("Engine compatibility").collapsible(false).show(egui_ctx,|ui|{
                            ui.label(notice);if ui.button("OK").clicked(){front.notice=None;}
                        });
                    }
                });
                if let Some(output)=ui.take_output(){ctx.submit_egui_frame(ui.egui_ctx.clone(),output);}
            }
            if let Some(settings)=front.settings.as_mut(){outputs.setting_changes.append(&mut settings.changes);}
            if front.settings.as_ref().is_some_and(|s|s.closed){front.settings=None;front.dirty=true;}
            crate::apply_debug_ui_outputs(&mut self.world,outputs,&mut self.debug_ui_refresh_entities,self.debug_ui.as_mut());
            front.draw(ctx)?;
            Ok(std::mem::replace(&mut front.action,Action::None))
        })();
        match result {
            Ok(Action::Options)=>{
                front.focus(None);
                match options::OptionsMenu::new(&front.assets,front.menu.screen()) {
                    Ok(settings)=>front.settings=Some(settings),
                    Err(error)=>front.notice=Some(format!("Original Options unavailable: {error:#}")),
                }
                self.front_end=Some(front);
            }
            Ok(Action::New)=>{
                match saves::new_game_start(&crate::cli_args::effective_args()) {
                    Ok((args,mut pose))=>{
                        self.close_front_end(&mut front);
                        self.world.insert_resource(SceneStartArgs(args));
                        self.world.insert_resource(crate::components::ExplicitStartPin(true));
                        let controller=byroredux_physics::CharacterController::HUMAN;
                        pose.position[1]=crate::scene::character_spawn_center_y(&self.world,pose.position[1],controller);
                        self.camera_pos_override=Some((pose.position[0],pose.position[1]+controller.eye_height,pose.position[2]));
                        let forward=crate::systems::camera_look_rotation(pose.yaw,pose.pitch)*-byroredux_core::math::Vec3::Z;
                        self.camera_forward_override=Some(forward.into());
                        self.setup_scene();
                        self.camera_pos_override=None;
                        self.camera_forward_override=None;
                        crate::save_io::apply_player_pose(&mut self.world,&pose);
                        if let Err(error)=crate::extensions::queue_session_event(&self.world,byroredux_sdk::event::SessionEvent {
                            phase:byroredux_sdk::event::SessionPhase::NewGame,slot:None,
                        }) {log::warn!("new-game lifecycle event: {error}");}
                        crate::sync_camera_setting(&self.world);
                        self.scheduler.run(&self.world,0.0);
                        log::info!("front-end: new Oblivion prison session; character creation/tutorial parity incomplete");
                    }
                    Err(error)=>{front.notice=Some(format!("New game refused: {error:#}"));self.front_end=Some(front);}
                }
            }
            Ok(Action::Load(slot))=>{
                match saves::prepare_load(&mut self.world,slot) {
                    Ok(())=>{
                        self.close_front_end(&mut front);
                        self.world.insert_resource(crate::components::ExplicitStartPin(true));
                        self.setup_scene();
                        crate::save_io::execute_pending_save_loads(&mut self.world,self.renderer.as_mut().unwrap(),&mut self.streaming);
                        crate::sync_camera_setting(&self.world);
                        self.scheduler.run(&self.world,0.0);
                        self.last_frame=Instant::now();
                        log::info!("front-end: committed engine save slot {slot}; world simulation remains held for restore");
                    }
                    Err(error)=>{front.notice=Some(format!("Load refused: {error:#}"));self.front_end=Some(front);}
                }
            }
            Ok(Action::Quit)=>{self.front_end=Some(front);self.shutdown(event_loop);}
            Ok(Action::None)=>self.front_end=Some(front),
            Err(error)=>{log::error!("Original front end failed: {error:#}");self.front_end=Some(front);self.shutdown(event_loop);}
        }
        self.last_frame=Instant::now();
        true
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn options_return_clears_status_and_reopening_keeps_text_readable() {
        for width in [640.0,1280.0] {
            let ctx=egui::Context::default();
            let status="Mouse sensitivity, invert Y and keyboard bindings are supported. Controller options are not yet supported.";
            // Open the empty Options landing page, show a status, Return to
            // the main menu, then reopen: exercise egui's remembered Area size.
            for message in [Some(""),Some(status),None,Some(""),Some(status),None] {
                for _ in 0..2 {
                    ctx.begin_pass(egui::RawInput {
                        screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(width,720.0))),
                        ..Default::default()
                    });
                    show_options_status(&ctx,message);
                    let output=ctx.end_pass();
                    let texts:Vec<_>=output.shapes.iter().filter_map(|shape|{
                        if let egui::Shape::Text(text)=&shape.shape {Some(text)}else{None}
                    }).collect();
                    if message.is_none_or(|s|s.is_empty()) {
                        assert!(texts.is_empty(),"Return must leave no status text on the main menu");
                    } else if let Some(text)=texts.first() {
                        assert_eq!(text.galley.text(),status);
                        assert!(text.galley.rect.width()>200.0,"status collapsed into a column");
                        assert!(text.galley.rect.width()<=width-32.0);
                        assert!(text.galley.rect.height()<80.0,"status obscures the menu vertically");
                    }
                }
            }
        }
    }
}
