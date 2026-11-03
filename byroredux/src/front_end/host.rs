//! Native host glue. Menu/world lifetimes remain separate until a validated load.
use super::*;
use byroredux_debug_ui::{egui, panels::{PanelSnapshot, PanelOutputs, GameMenuPage}};
use winit::{event::WindowEvent,event_loop::ActiveEventLoop};

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
        let native=self.front_end.as_ref().is_some_and(|f|f.settings.visible||f.show_saves);
        let consumed=self.debug_ui.as_mut().is_some_and(|ui|ui.on_window_event(win,event).consumed);
        if let WindowEvent::Resized(size)=event {
            if size.width>0 && size.height>0 {
                if let Err(error)=self.renderer.as_mut().unwrap().recreate_swapchain([size.width,size.height]) {
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
    pub(crate) fn step_front_end(&mut self,event_loop:&ActiveEventLoop)->bool {
        let Some(mut front)=self.front_end.take() else{return false;};
        #[cfg(feature="debug-server")]
        self.scheduler.run_exclusive_named(&self.world,0.0,byroredux_debug_server::DRAIN_SYSTEM_NAME);
        let result=(||->Result<Action>{
            let ctx=self.renderer.as_mut().unwrap();
            front.tick(&self.world,ctx)?;
            let mut outputs=PanelOutputs::default();
            if let (Some(ui),Some(win))=(self.debug_ui.as_mut(),self.window.as_ref()) {
                let snapshot=PanelSnapshot {settings:self.world.resource::<byroredux_core::settings::SettingsRegistry>().entries().cloned().collect(),..Default::default()};
                ui.run_front_end(win,|egui_ctx|{
                    if front.phase==Phase::Menu {
                        egui::Area::new(egui::Id::new("front_end_compatibility")).anchor(egui::Align2::CENTER_BOTTOM,egui::vec2(0.0,-12.0))
                            .interactable(false).show(egui_ctx,|ui|{
                                ui.label("COMPATIBILITY: New Game and Bethesda saves unsupported. Options: engine settings.");
                            });
                    }
                    if egui_ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        front.settings.visible=false;front.show_saves=false;front.notice=None;
                    }
                    if front.settings.visible {
                        byroredux_debug_ui::panels::draw_game_menu(egui_ctx,&snapshot,&mut front.settings,&mut outputs);
                        if front.settings.page!=GameMenuPage::Settings || outputs.resume_game {front.settings.visible=false;}
                    }
                    if front.show_saves {
                        egui::Window::new("Engine saves — original Bethesda saves unsupported").collapsible(false).show(egui_ctx,|ui|{
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
            crate::apply_debug_ui_outputs(&mut self.world,outputs,&mut self.debug_ui_refresh_entities,self.debug_ui.as_mut());
            front.draw(ctx)?;
            Ok(std::mem::replace(&mut front.action,Action::None))
        })();
        match result {
            Ok(Action::Load(slot))=>{
                match saves::prepare_load(&mut self.world,slot) {
                    Ok(())=>{
                        self.world.resource_mut::<AudioWorld>().stop_music(0.0);
                        self.world.resource_mut::<AudioWorld>().stop_presentation();
                        front.movie.take();
                        if let Some(texture)=front.texture.take(){let ctx=self.renderer.as_mut().unwrap();ctx.texture_registry.drop_texture(&ctx.device,texture);}
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
