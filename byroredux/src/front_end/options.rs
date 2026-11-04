//! Original Oblivion Options MenuXml, backed by the existing settings registry.
use super::*;
use byroredux_core::settings::{SettingChange, SettingControl, SettingEntry, SettingValue, SettingsRegistry};
use byroredux_menuxml::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page { Options, Gameplay, Video, Audio, Controls, Bindings }
impl Page {
    fn resource(self)->(&'static str,&'static str) {
        match self {
            Self::Options=>("options_menu","OptionsMenu"), Self::Gameplay=>("gameplay_menu","GameplayMenu"),
            Self::Video=>("video_menu","VideoMenu"), Self::Audio=>("audio_menu","AudioMenu"),
            Self::Controls|Self::Bindings=>("controls_menu","ControlsMenu"),
        }
    }
}

pub(super) struct OptionsMenu {
    renderer:MenuRenderer, page:Page, next:Option<Page>, targets:Vec<HitTarget>,
    focus:Option<String>, entries:Vec<SettingEntry>, pub changes:Vec<SettingChange>,
    pub closed:bool, pub status:String, dragging:bool, binding:Option<String>, scroll:usize,
}
impl OptionsMenu {
    pub fn new(assets:&crate::hud::HudAssets,screen:ScreenTraits)->Result<Self> {
        let renderer=MenuRenderer::load(assets,"menus\\options\\options_menu.xml",screen)?;
        let mut menu=Self {renderer,page:Page::Options,next:None,targets:Vec::new(),focus:None,
            entries:Vec::new(),changes:Vec::new(),closed:false,status:String::new(),dragging:false,binding:None,scroll:0};
        menu.open(Page::Options,assets)?;
        Ok(menu)
    }
    fn open(&mut self,page:Page,assets:&crate::hud::HudAssets)->Result<()> {
        let (file,root)=page.resource();
        self.renderer=MenuRenderer::load(assets,&format!("menus\\options\\{file}.xml"),self.renderer.screen())?;
        self.renderer.set_override(root,"alpha",255.0);
        self.page=page;self.focus=None;self.dragging=false;self.binding=None;self.scroll=0;
        // Shipped long-button prefabs carry controller glyph hints. PC uses
        // the authored text-button style instead of Xbox glyph+label mode.
        for tile in ["gameplay_defaults_button","gameplay_return_button","video_defaults_button","video_return_button",
            "audio_defaults_button","audio_return_button","controls_defaults_button","controls_return_button","controls_return_button_2","controls_more_button"] {
            self.renderer.set_override(tile,"user20",1.0);
        }
        self.status=match page {
            Page::Gameplay=>"Crosshair is supported. Other gameplay options are not yet supported.",
            Page::Video=>"Original video settings are not yet supported by this renderer.",
            Page::Audio=>"Audio volume options are not yet supported.",
            Page::Controls=>"Mouse sensitivity, invert Y and keyboard bindings are supported. Controller options are not yet supported.",
            Page::Bindings=>"Select an action, then press a key. Scroll for more actions.",
            Page::Options=>"",
        }.into();
        match page {
            Page::Options=>self.renderer.set_override(root,"user0",0.0),
            Page::Gameplay=>{
                for tile in ["gameplay_general_subtitles_button","gameplay_dialogue_subtitles_button",
                    "gameplay_rest_save_button","gameplay_wait_save_button","gameplay_travel_save_button"] {
                    self.renderer.set_override_str(tile,"string","Unsupported");
                    self.renderer.set_override(tile,"alpha",128.0);
                    self.renderer.set_override(tile,"target",0.0);
                }
                self.renderer.set_override("gameplay_difficulty_slider","user13",0.0);
                self.renderer.set_override("gameplay_difficulty_slider_marker","target",0.0);
            }
            Page::Audio=>{self.renderer.set_override("audio_defaults_button","user1",0.0);
                for tile in ["audio_master_slider","audio_voice_slider","audio_effect_slider","audio_foot_slider","audio_music_slider"] {
                self.renderer.set_override(tile,"user13",0.0);
            }},
            Page::Controls|Page::Bindings=>{
                let bindings=page==Page::Bindings;
                self.renderer.set_override("controls_page_1","depth",3.0);
                self.renderer.set_override("controls_page_2","depth",3.0);
                self.renderer.set_override(root,"user0",if bindings{2.0}else{1.0});
                self.renderer.set_override("controls_page_1","visible",2.0);
                self.renderer.set_override("controls_page_2","visible",2.0);
                for tile in ["controls_swap_button","controls_left_stick_label","controls_right_stick_label","controls_left_stick_setting","controls_right_stick_setting"] {
                    self.renderer.set_override(tile,"visible",0.0);
                }
                self.renderer.set_override("controls_sensitivity_slider","target",2.0);
                self.renderer.set_override_str(root,"user5","Keyboard");
                if bindings {
                    self.renderer.set_override("controls_list_pane","target",2.0);
                    for i in 0..6 {
                        let name=format!("binding_{i}");
                        self.renderer.instantiate_template("controls_template","controls_list_pane",&name)?;
                        for (property,value) in [("user0",i as f32),("height",64.0),("red",117.0),("green",59.0),("blue",33.0)] {
                            self.renderer.set_override(&name,property,value);
                        }
                    }
                }
            }
            Page::Video=>{
                self.renderer.set_override("video_defaults_button","user1",0.0);
                for tile in ["video_resolution_rect","video_resolution_left","video_resolution_right"] {self.renderer.set_override(tile,"target",0.0);}
                self.renderer.set_override_str(root,"user4","Current window");
                self.renderer.set_override_str(root,"user5","Engine controlled");
                for tile in ["video_gamma_slider","video_tree_lod_slider","video_actor_lod_slider","video_item_lod_slider",
                    "video_object_lod_slider","video_grass_slider","video_viewdist_slider","video_intshad_slider","video_extshad_slider","video_speclight_slider"] {
                    self.renderer.set_override(tile,"user13",0.0);
                }
                for tile in ["video_texsize_button","video_LOD_button","video_distbuild_button","video_disttrees_button",
                    "video_self_shadows_button","video_grass_shadows_button","video_canopy_button","video_shadow_filter_button",
                    "video_hdr_button","video_bloom_button","video_water_detail_button","video_water_reflections_button",
                    "video_water_displacement_button","video_window_reflections_button","video_hair_detail_button","video_blood_button"] {
                    self.renderer.set_override_str(tile,"string","Unsupported");
                    self.renderer.set_override(tile,"alpha",128.0);
                    self.renderer.set_override(tile,"target",0.0);
                }
            }
        }
        Ok(())
    }
    fn bindings(&self)->Vec<&SettingEntry> {
        self.entries.iter().filter(|e|e.id.starts_with("controls.bind.")&&e.id!="controls.bind.vats").collect()
    }
    fn entry(&self,id:&str)->Option<&SettingEntry> {self.entries.iter().find(|e|e.id==id)}
    fn change(&mut self,id:&str,value:SettingValue) {
        if let Some(entry)=self.entries.iter_mut().find(|e|e.id==id) {
            entry.value=value.clone();self.changes.push(SettingChange::new(id,value));
        }
    }
    fn toggle(&mut self,id:&str) {
        if let Some(SettingValue::Bool(v))=self.entry(id).map(|e|e.value.clone()) {self.change(id,SettingValue::Bool(!v));}
    }
    fn defaults(&mut self) {
        let ids:Vec<_>=self.entries.iter().filter(|e|match self.page {
            Page::Gameplay=>e.id==byroredux_core::settings::builtin::SHOW_CROSSHAIR_SETTING_ID,
            Page::Controls=>[crate::interaction::MOUSE_SENSITIVITY_SETTING_ID,crate::interaction::INVERT_LOOK_Y_SETTING_ID].contains(&e.id.as_str()),
            Page::Bindings=>e.id.starts_with("controls.bind.")&&e.id!="controls.bind.vats", _=>false,
        }).map(|e|(e.id.clone(),e.default.clone())).collect();
        for (id,value) in ids {self.change(&id,value);}
    }
    fn back(&mut self) {
        if self.binding.take().is_some(){return;}
        if self.page==Page::Options {self.closed=true;} else {self.next=Some(if self.page==Page::Bindings{Page::Controls}else{Page::Options});}
    }
    fn activate(&mut self,name:&str) {
        match name {
            "options_return_button"|"gameplay_return_button"|"audio_return_button"|"video_return_button"|"controls_return_button"|"controls_return_button_2"=>self.back(),
            "options_gameplay_button"=>self.next=Some(Page::Gameplay),
            "options_video_button"=>self.next=Some(Page::Video),
            "options_audio_button"=>self.next=Some(Page::Audio),
            "options_controls_button"=>self.next=Some(Page::Controls),
            "controls_more_button"=>self.next=Some(Page::Bindings),
            "vertical_scroll_top"=>self.scroll=self.scroll.saturating_sub(1),
            "vertical_scroll_bottom"=>self.scroll=(self.scroll+1).min(if self.page==Page::Video{20}else{self.bindings().len().saturating_sub(6)}),
                        "controls_invert_y_button"=>self.toggle(crate::interaction::INVERT_LOOK_Y_SETTING_ID),
            "gameplay_crosshair_button"=>self.toggle(byroredux_core::settings::builtin::SHOW_CROSSHAIR_SETTING_ID),
            "gameplay_defaults_button"|"controls_defaults_button"=>self.defaults(),
            _=>{
                if let Some(i)=name.strip_prefix("binding_").and_then(|s|s.parse::<usize>().ok()) {
                    self.binding=self.bindings().get(self.scroll+i).map(|e|e.id.clone());
                    self.status="Press a key; Escape cancels. Existing bindings swap when necessary.".into();
                }
            }
        }
    }
    fn focus(&mut self,name:Option<String>) {
        if self.focus==name{return;}
        for target in &self.targets {self.renderer.set_override(&target.name,"mouseover",if Some(&target.name)==name.as_ref(){1.0}else{0.0});}
        self.focus=name;
    }
    fn slider(&self)->Option<Rect> {self.targets.iter().find(|t|t.name=="controls_sensitivity_slider").map(|t|t.rect)}
    fn set_sensitivity(&mut self,fraction:f32) {
        let id=crate::interaction::MOUSE_SENSITIVITY_SETTING_ID;
        if let Some(SettingControl::Slider{min,max,step,..})=self.entry(id).map(|e|e.control.clone()) {
            let value=slider_value(fraction,min,max,step);
            self.change(id,SettingValue::Number(value));
        }
    }
    pub fn input(&mut self,event:UiInputEvent) {
        if let UiInputEvent::KeyDown{ref key}=event {
            if key.logical_key==UiLogicalKey::Named(UiNamedKey::Escape){self.back();return;}
            if let Some(id)=self.binding.clone() {
                let physical=format!("{:?}",key.physical_key).to_ascii_lowercase();
                let value=self.entry(&id).and_then(|e|if let SettingControl::Choice{options}=&e.control {
                    options.iter().find(|o|o.value.replace('_',"")==physical).map(|o|o.value.clone())
                }else{None});
                if let Some(value)=value {self.change(&id,SettingValue::Choice(value));self.binding=None;self.status="Binding updated.".into();}
                else {self.status="This key is unavailable; choose another key or press Escape.".into();}
                return;
            }
        }
        match event {
            UiInputEvent::MouseMove{x,y}=>{
                if self.dragging {if let Some(rect)=self.slider(){self.set_sensitivity((x as f32-rect.x)/rect.w);}}
                let hit=self.targets.iter().rev().find(|t|contains(t.rect,x,y)).map(|t|t.name.clone());self.focus(hit);
            }
            UiInputEvent::MouseDown{button:UiMouseButton::Left,x,y}=>{
                if self.page==Page::Controls {
                    if let Some(rect)=self.slider() {
                        if contains(Rect{x:rect.x-50.0,y:rect.y-16.0,w:rect.w+101.0,h:48.0},x,y) {
                            self.dragging=true;self.set_sensitivity((x as f32-rect.x)/rect.w);return;
                        }
                    }
                }
                if let Some(name)=self.targets.iter().rev().find(|t|contains(t.rect,x,y)).map(|t|t.name.clone()){self.activate(&name);}
            }
            UiInputEvent::MouseUp{button:UiMouseButton::Left,..}=>self.dragging=false,
            UiInputEvent::MouseLeave|UiInputEvent::FocusLost=>{self.dragging=false;self.focus(None);}
            UiInputEvent::MouseWheel{delta}=>{
                let amount=match delta {byroredux_ui::UiMouseWheelDelta::Lines(y)=>y,byroredux_ui::UiMouseWheelDelta::Pixels(y)=>y/40.0};
                if matches!(self.page,Page::Bindings|Page::Video) {
                    let max=if self.page==Page::Video{20}else{self.bindings().len().saturating_sub(6)};
                    self.scroll=if amount<0.0{(self.scroll+1).min(max)}else{self.scroll.saturating_sub(1)};
                }
            }
            UiInputEvent::KeyDown{key}=>match key.logical_key {
                UiLogicalKey::Named(UiNamedKey::Enter)|UiLogicalKey::Character(' ')=>if let Some(name)=self.focus.clone(){self.activate(&name);},
                UiLogicalKey::Named(UiNamedKey::ArrowLeft|UiNamedKey::ArrowRight) if self.focus.as_deref()==Some("controls_sensitivity_slider")=>{
                    if let Some(entry)=self.entry(crate::interaction::MOUSE_SENSITIVITY_SETTING_ID) {
                        if let (SettingValue::Number(v),SettingControl::Slider{min,max,step,..})=(&entry.value,&entry.control) {
                            let direction=if key.logical_key==UiLogicalKey::Named(UiNamedKey::ArrowLeft){-1.0}else{1.0};
                            self.set_sensitivity((*v+direction*step-min)/(max-min));
                        }
                    }
                }
                UiLogicalKey::Named(UiNamedKey::ArrowUp|UiNamedKey::ArrowDown|UiNamedKey::Tab)=>{
                    let names:Vec<_>=self.targets.iter().filter(|t|self.actionable(&t.name)).map(|t|t.name.clone()).collect();
                    if !names.is_empty(){let backwards=key.logical_key==UiLogicalKey::Named(UiNamedKey::ArrowUp);
                        let step=if backwards{names.len()-1}else{1};
                        let i=self.focus.as_ref().and_then(|n|names.iter().position(|s|s==n)).map_or(if backwards{names.len()-1}else{0},|i|(i+step)%names.len());
                        self.focus(Some(names[i].clone()));}
                }
                _=>{}
            },_=>{}
        }
    }
    fn actionable(&self,name:&str)->bool {
        name.starts_with("options_")||name.contains("return_button")||name.starts_with("binding_")||
            matches!(name,"gameplay_crosshair_button"|"controls_invert_y_button"|"controls_sensitivity_slider"|"controls_map_button"|"controls_more_button")||
            (name.contains("defaults_button")&&matches!(self.page,Page::Gameplay|Page::Controls|Page::Bindings))
    }
    pub fn frame(&mut self,assets:&crate::hud::HudAssets,screen:ScreenTraits,settings:&SettingsRegistry)->Result<Rgba8> {
        self.renderer.set_screen(screen);
        if let Some(page)=self.next.take(){self.open(page,assets)?;}
        self.entries=settings.entries().cloned().collect();
        for (tile,id) in [("gameplay_crosshair_button",byroredux_core::settings::builtin::SHOW_CROSSHAIR_SETTING_ID),
            ("controls_invert_y_button",crate::interaction::INVERT_LOOK_Y_SETTING_ID)] {
            if let Some(SettingValue::Bool(v))=self.entry(id).map(|e|e.value.clone()) {self.renderer.set_override_str(tile,"string",if v{"On"}else{"Off"});}
        }
        if let Some(SettingEntry{value:SettingValue::Number(v),control:SettingControl::Slider{min,max,..},..})=self.entry(crate::interaction::MOUSE_SENSITIVITY_SETTING_ID) {
            let percent=100.0*(v-min)/(max-min);
            self.renderer.set_override("controls_sensitivity_slider","user5",percent);
        }
        if self.page==Page::Bindings {
            self.renderer.set_override("controls_vertical_scroll","user2",self.bindings().len().saturating_sub(6) as f32);
            self.renderer.set_override("controls_vertical_scroll","user7",0.0);
            self.renderer.set_override("vertical_scroll_marker","_value_scrolled_to",self.scroll as f32);
            let rows:Vec<_>=self.bindings().iter().skip(self.scroll).take(6).map(|e|{
                let label=if let (SettingValue::Choice(value),SettingControl::Choice{options})=(&e.value,&e.control) {
                    options.iter().find(|o|&o.value==value).map(|o|o.label.clone()).unwrap_or_default()
                }else{String::new()};(e.label.clone(),label)
            }).collect();
            for i in 0..6 {let name=format!("binding_{i}");self.renderer.set_override(&name,"visible",if i<rows.len(){2.0}else{0.0});
                if let Some((action,key))=rows.get(i){self.renderer.set_override_str(&name,"user1",action);self.renderer.set_override_str(&name,"user3",key);}}
        }
        if self.page==Page::Video {
            self.renderer.set_override("video_vertical_scroll","user7",self.scroll as f32);
            self.renderer.set_override("vertical_scroll_marker","_value_scrolled_to",self.scroll as f32);
        }
        self.targets=self.renderer.hit_targets();
        if self.page==Page::Bindings {
            if let Some(pane)=self.targets.iter().find(|t|t.name=="controls_list_pane").map(|t|t.rect) {
                let row_height=pane.h/6.0;
                for i in 0..6 {self.renderer.set_override(&format!("binding_{i}"),"height",row_height);}
                for i in 0..6.min(self.bindings().len().saturating_sub(self.scroll)) {
                    self.targets.push(HitTarget{tile:usize::MAX,id:100+i as i32,name:format!("binding_{i}"),
                        rect:Rect{x:pane.x,y:pane.y+row_height*i as f32,w:pane.w,h:row_height}});
                }
            }
        }
        // Unsupported authored fields remain visible, but must not gain a
        // hover highlight or receive activation as if they were implemented.
        self.targets=self.targets.iter().filter(|t|self.actionable(&t.name)||
            matches!(t.name.as_str(),"vertical_scroll_top"|"vertical_scroll_bottom")).cloned().collect();
        let (width,height)=self.renderer.frame_size();
        Ok(Rgba8{width,height,pixels:self.renderer.render_frame(assets).to_vec()})
    }
}
fn contains(rect:Rect,x:f64,y:f64)->bool {x>=rect.x as f64&&y>=rect.y as f64&&x<(rect.x+rect.w)as f64&&y<(rect.y+rect.h)as f64}
fn slider_value(fraction:f32,min:f32,max:f32,step:f32)->f32 {
    (min+((fraction.clamp(0.0,1.0)*(max-min))/step).round()*step).clamp(min,max)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_options_slider_clamps_and_uses_registry_steps() {
        assert_eq!(slider_value(-1.0,0.1,4.0,0.05),0.1);
        assert_eq!(slider_value(2.0,0.1,4.0,0.05),4.0);
        assert!((slider_value(0.5,0.1,4.0,0.05)-2.05).abs()<0.0001);
    }
    #[test]
    #[ignore="requires the user's installed original Oblivion data"]
    fn front_end_original_options_pages_and_settings_smoke() {
        let data=std::env::var_os("BYROREDUX_OBLIVION_DATA").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data"));
        let assets=crate::hud::HudAssets::oblivion(&data).unwrap();
        let mut registry=SettingsRegistry::default();
        byroredux_debug_ui::register_builtin_settings(&mut registry).unwrap();
        crate::interaction::register_input_settings(&mut registry).unwrap();
        let screen=ScreenTraits::new(1706.0,960.0);
        let mut menu=OptionsMenu::new(&assets,screen).unwrap();
        let output=std::env::temp_dir().join("byroredux-original-options-frames");
        std::fs::create_dir_all(&output).unwrap();
        for page in [Page::Options,Page::Gameplay,Page::Video,Page::Audio,Page::Controls,Page::Bindings] {
            menu.open(page,&assets).unwrap();
            let frame=menu.frame(&assets,screen,&registry).unwrap();
            assert!(frame.pixels.chunks_exact(4).any(|p|p[3]>0));
            println!("{page:?} targets: {:?}",menu.targets.iter().map(|t|(&t.name,t.rect)).collect::<Vec<_>>());
            assert!(menu.targets.iter().any(|t|t.name.contains("return_button")),"{page:?} has no Return target");
            image::save_buffer(output.join(format!("{page:?}.png")),&frame.pixels,frame.width,frame.height,image::ColorType::Rgba8).unwrap();
        }
        assert!(menu.targets.iter().any(|t|t.name=="binding_0"));
        menu.activate("binding_0");
        assert!(menu.binding.is_some());
        menu.input(UiInputEvent::KeyDown{key:byroredux_ui::UiKeyDescriptor {
            physical_key:byroredux_ui::UiPhysicalKey::KeyW,logical_key:UiLogicalKey::Character('w'),key_location:byroredux_ui::UiKeyLocation::Standard,
        }});
        assert_eq!(menu.changes.last().unwrap().value,SettingValue::Choice("key_w".into()));
        menu.activate("binding_0");
        menu.back();
        assert!(menu.binding.is_none());
        assert!(menu.next.is_none(),"canceling a binding must stay on the page");
        menu.open(Page::Gameplay,&assets).unwrap();
        menu.frame(&assets,screen,&registry).unwrap();
        menu.activate("gameplay_crosshair_button");
        assert_eq!(menu.changes.last().unwrap().value,SettingValue::Bool(false));
        for change in menu.changes.drain(..){registry.set(&change.id,change.value).unwrap();}
        menu.frame(&assets,screen,&registry).unwrap();
        menu.defaults();
        assert_eq!(menu.changes.last().unwrap().value,SettingValue::Bool(true));
        menu.open(Page::Controls,&assets).unwrap();
        menu.frame(&assets,screen,&registry).unwrap();
        assert!(menu.targets.iter().any(|t|t.name=="controls_more_button"));
        let rect=menu.slider().expect("original sensitivity hit rectangle");
        menu.input(UiInputEvent::MouseDown{button:UiMouseButton::Left,x:(rect.x+rect.w)as f64,y:rect.y as f64});
        assert_eq!(menu.changes.last().unwrap().value,SettingValue::Number(4.0));
        menu.input(UiInputEvent::MouseUp{button:UiMouseButton::Left,x:0.0,y:0.0});
        menu.activate("controls_invert_y_button");
        assert_eq!(menu.changes.last().unwrap().value,SettingValue::Bool(true));
    }
}
