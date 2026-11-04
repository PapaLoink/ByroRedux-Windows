//! Original Oblivion front end. Proprietary resources are loaded in place.
mod config;
mod media;
mod saves;
mod host;
mod presentation;
mod options;
pub(crate) use saves::SceneStartArgs;
use anyhow::{bail, Result};
use byroredux_audio::AudioWorld;
use byroredux_menuxml::{MenuRenderer, ScreenTraits, tex::Rgba8, layout::HitTarget};
use byroredux_ui::{UiInputEvent, UiLogicalKey, UiNamedKey, UiMouseButton};
use std::time::Instant;
use config::Config;
use media::{Decoder, Event, Metadata};

pub(crate) fn requested(args: &[String]) -> bool {
    args.iter().any(|a| a=="--oblivion-main-menu") &&
        !args.iter().any(|a| ["--bench-frames","--studio","--cmd","--list-cells","--load","--menu","--cell","--grid"].contains(&a.as_str()))
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
enum Phase { Startup(usize), Title, Menu, Credits }
fn next_phase(phase: Phase, count: usize) -> Phase {
    match phase { Phase::Startup(i) if i+1<count => Phase::Startup(i+1),
        Phase::Startup(_) => Phase::Title, Phase::Title|Phase::Credits|Phase::Menu => Phase::Menu }
}
#[derive(Debug,Clone,Copy)]
pub(super) enum Action { None, New, Options, Load(u32), Quit }

struct Movie {
    decoder: Decoder, metadata: Option<Metadata>, sound: Option<byroredux_audio::Sound>,
    started: Option<Instant>, pending: Option<(f64,Vec<u8>)>, ended: bool, audio_clock: bool,
}
impl Movie {
    fn new(path: &std::path::Path) -> Result<Self> {
        log::info!("front-end: movie {}",path.display());
        Ok(Self { decoder:Decoder::start(path)?, metadata:None, sound:None, started:None,
            pending:None, ended:false, audio_clock:false })
    }
    fn tick(&mut self, audio: &mut AudioWorld) -> Result<(Option<Rgba8>,bool)> {
        let mut frame=None;
        for _ in 0..8 {
            if let Some((pts,_))=&self.pending {
                if self.started.is_none() {
                    self.started=Some(Instant::now());
                    if let Some(sound)=self.sound.take() {
                        audio.play_presentation(sound).map_err(anyhow::Error::msg)?;
                        self.audio_clock=true;
                    }
                }
                let time=if self.audio_clock { audio.presentation_position().unwrap_or_else(|| self.started.unwrap().elapsed().as_secs_f64()) }
                    else { self.started.unwrap().elapsed().as_secs_f64() };
                if *pts>time { break; }
                let (_,pixels)=self.pending.take().unwrap();
                let meta=self.metadata.unwrap();
                frame=Some(Rgba8 { width:meta.width,height:meta.height,pixels });
            }
            match self.decoder.poll() {
                Some(Event::Ready(meta,sound)) => { self.metadata=Some(meta);self.sound=sound; }
                Some(Event::Frame(pts,pixels)) => self.pending=Some((pts,pixels)),
                Some(Event::End) => { self.ended=true;break; }
                Some(Event::Error(error)) => bail!("original movie playback: {error}"),
                None=>break,
            }
        }
        let finished=self.ended && self.pending.is_none() && self.metadata.zip(self.started)
            .is_some_and(|(meta,start)| start.elapsed().as_secs_f64()>=meta.duration);
        Ok((frame,finished))
    }
}

pub(crate) struct FrontEnd {
    config: Config, assets: crate::hud::HudAssets, menu: MenuRenderer,
    phase:Phase, movie:Option<Movie>, video:Option<Rgba8>, menu_image:Option<Rgba8>,
    targets:Vec<HitTarget>, focus:Option<i32>, dirty:bool, music_started:bool,
    size:(u32,u32), texture:Option<u32>, saves:Vec<saves::SaveSlot>,
    settings:Option<options::OptionsMenu>, show_saves:bool, show_new:bool,
    notice:Option<String>, action:Action,
}
impl FrontEnd {
    fn new(data:&std::path::Path, world:&byroredux_core::ecs::World) -> Result<Self> {
        let config=Config::load(data)?;
        let assets=crate::hud::HudAssets::oblivion(&config.data)?;
        let mut menu=MenuRenderer::load(&assets,"menus\\options\\main_menu.xml",ScreenTraits::new(1280.0,960.0))?;
        menu.set_override("MainMenu","user0",2.0);
        menu.set_override("MainMenu","user1",2.0);
        menu.set_override("MainMenu","user2",0.0);
        menu.set_override("MainMenu","user3",0.0);
        menu.set_override("main_new","visible",2.0);
        let saves=saves::slots(world);
        menu.set_override("main_continue","visible",if saves.is_empty(){0.0}else{2.0});
        menu.set_override("main_load","visible",2.0);
        let movie=Some(Movie::new(&config.startup[0])?);
        log::warn!("front-end compatibility: New starts the original prison scene; character creation/tutorial and original Bethesda saves remain incomplete; Options uses engine settings");
        Ok(Self { config,assets,menu,phase:Phase::Startup(0),movie,video:None,menu_image:None,targets:Vec::new(),focus:None,
            dirty:true,music_started:false,size:(0,0),texture:None,saves,
            settings:Default::default(),show_saves:false,show_new:false,notice:None,action:Action::None })
    }
    fn advance(&mut self,audio:&mut AudioWorld)->Result<()> {
        let loop_frame = if self.phase==Phase::Menu { self.video.take() } else { None };
        audio.stop_presentation(); self.movie.take();
        self.phase=next_phase(self.phase,self.config.startup.len());
        self.open_phase(audio)?;
        if loop_frame.is_some() {self.video=loop_frame;}
        Ok(())
    }
    fn open_phase(&mut self,audio:&mut AudioWorld)->Result<()> {
        audio.stop_presentation();
        self.video=None;
        let path=match self.phase { Phase::Startup(i)=>&self.config.startup[i],Phase::Title=>&self.config.title,
            Phase::Menu=>&self.config.map,Phase::Credits=>&self.config.credits };
        self.movie=Some(Movie::new(path)?);
        if matches!(self.phase,Phase::Title|Phase::Menu) && !self.music_started {
            let sound=byroredux_audio::load_streaming_sound_from_file(&self.config.music)?;
            if !audio.is_active() { bail!("audio device unavailable for original title music"); }
            audio.play_music(sound,0.6,0.0,true); self.music_started=true;
        }
        if self.phase==Phase::Credits { audio.stop_music(0.0);self.music_started=false; }
        log::info!("front-end: phase {:?}",self.phase);
        self.dirty=true;
        Ok(())
    }
    fn choose(&mut self,id:i32) {
        self.action=match id { 3=>{self.show_new=true;Action::None}, 2=>self.saves.first().map_or(Action::None,|s|Action::Load(s.number)),
            4=>{self.show_saves=true;Action::None},
            5=>Action::Options,
            6=>Action::None,7=>Action::Quit,_=>Action::None };
        if id==6 { self.phase=Phase::Credits;self.movie.take(); }
    }
    fn focus(&mut self,id:Option<i32>) {
        if self.focus==id{return;}
        self.focus=id;
        for target in &self.targets {self.menu.set_override(&target.name,"mouseover",if Some(target.id)==id{1.0}else{0.0});}
        self.dirty=true;
    }
    pub(super) fn input(&mut self,event:UiInputEvent,audio:&mut AudioWorld)->Result<()> {
        if let Some(settings)=self.settings.as_mut(){settings.input(event);return Ok(());}
        if self.show_saves || self.show_new || self.notice.is_some() {
            if matches!(event,UiInputEvent::KeyDown{ref key} if key.logical_key==UiLogicalKey::Named(UiNamedKey::Escape)) {
                self.show_saves=false;self.show_new=false;self.notice=None;
            }
            return Ok(());
        }
        if self.phase!=Phase::Menu {
            if matches!(event,UiInputEvent::MouseDown{button:UiMouseButton::Left,..}) ||
                matches!(event,UiInputEvent::KeyDown{ref key} if (matches!(key.logical_key,UiLogicalKey::Named(UiNamedKey::Escape|UiNamedKey::Enter)) || key.logical_key==UiLogicalKey::Character(' '))) {
                self.advance(audio)?;
            }
            return Ok(());
        }
        match event {
            UiInputEvent::MouseMove{x,y}=> {
                let hit=self.targets.iter().rev().find(|t|x>=t.rect.x as f64&&y>=t.rect.y as f64&&x<(t.rect.x+t.rect.w)as f64&&y<(t.rect.y+t.rect.h)as f64).map(|t|t.id);
                self.focus(hit);
            }
            UiInputEvent::MouseDown{button:UiMouseButton::Left,x,y}=> {
                let hit=self.targets.iter().rev().find(|t|x>=t.rect.x as f64&&y>=t.rect.y as f64&&x<(t.rect.x+t.rect.w)as f64&&y<(t.rect.y+t.rect.h)as f64).map(|t|t.id);
                if let Some(id)=hit {self.choose(id);}
            }
            UiInputEvent::FocusLost|UiInputEvent::MouseLeave=>self.focus(None),
            UiInputEvent::KeyDown{key}=>match key.logical_key {
                UiLogicalKey::Named(UiNamedKey::Enter) | UiLogicalKey::Character(' ')=>if let Some(id)=self.focus{self.choose(id);},
                UiLogicalKey::Named(UiNamedKey::ArrowLeft|UiNamedKey::ArrowRight|UiNamedKey::Tab)=> {
                    if !self.targets.is_empty() {
                        let left=key.logical_key==UiLogicalKey::Named(UiNamedKey::ArrowLeft);
                        let i=self.targets.iter().position(|t|Some(t.id)==self.focus)
                            .map_or(if left{self.targets.len()-1}else{0},|i| {
                                (i+if left{self.targets.len()-1}else{1})%self.targets.len()
                            });
                        self.focus(Some(self.targets[i].id));
                    }
                },_=>{}
            },_=>{}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the user's installed original Oblivion data"]
    fn front_end_original_title_music_decodes_to_finite_non_silent_audio() {
        let data=std::env::var_os("BYROREDUX_OBLIVION_DATA").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data"));
        let config=Config::load(&data).unwrap();
        let bytes=std::fs::read(&config.music).unwrap();
        let sound=byroredux_audio::load_sound_from_bytes(bytes).expect("complete original title MP3 decode");
        assert_eq!(sound.sample_rate,44100);
        let duration=sound.frames.len() as f64 / sound.sample_rate as f64;
        // Installed TES4Title.mp3: ffprobe reports 110.942042 s. Allow encoder padding.
        assert!((110.0..112.0).contains(&duration),"decoded duration: {duration}");
        assert!(sound.frames.iter().all(|f|f.left.is_finite() && f.right.is_finite()));
        assert!(sound.frames.iter().any(|f|f.left.abs()>0.01 || f.right.abs()>0.01));
    }
    #[test]
    #[ignore = "requires the user's installed original Oblivion data"]
    fn front_end_original_menu_and_movie_resources_smoke() {
        let data=std::env::var_os("BYROREDUX_OBLIVION_DATA").map(std::path::PathBuf::from)
            .unwrap_or_else(||std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data"));
        let config=Config::load(&data).unwrap();
        let assets=crate::hud::HudAssets::oblivion(&config.data).unwrap();
        let mut menu=MenuRenderer::load(&assets,"menus\\options\\main_menu.xml",ScreenTraits::new(1706.0,960.0)).unwrap();
        menu.set_override("MainMenu","user3",0.0);
        menu.set_override("main_new","visible",2.0);
        menu.set_override("main_continue","visible",0.0);
        menu.set_override("main_load","visible",2.0);
        let targets=menu.hit_targets();
        assert_eq!(targets.iter().map(|t|t.id).collect::<Vec<_>>(),vec![3,4,5,6,7]);
        for t in &targets {assert!(t.rect.w>40.0&&t.rect.h>0.0);println!("authored target {} {:?}",t.id,t.rect);}
        let pixels=menu.render_frame(&assets);
        assert!(pixels.chunks_exact(4).any(|p|p[3]>0));
        let decoder=Decoder::start(&config.startup[0]).unwrap();
        let deadline=Instant::now()+std::time::Duration::from_secs(25);
        let mut first=false;
        while Instant::now()<deadline {
            match decoder.poll() {
                Some(Event::Frame(_,pixels))=>{assert!(!pixels.is_empty());first=true;break;},
                Some(Event::Error(e))=>panic!("{e}"),_=>std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
        assert!(first,"original movie produced no frame before the deadline");
        drop(decoder); // live decoder child must be cancelled and reaped
    }

    #[test]
    fn front_end_completion_and_skip_share_the_same_transition() {
        assert_eq!(next_phase(Phase::Startup(0),4),Phase::Startup(1));
        assert_eq!(next_phase(Phase::Startup(3),4),Phase::Title);
        assert_eq!(next_phase(Phase::Title,4),Phase::Menu);
        assert_eq!(next_phase(Phase::Menu,4),Phase::Menu);
        assert_eq!(next_phase(Phase::Credits,4),Phase::Menu);
    }
    #[test]
    fn front_end_does_not_intercept_direct_developer_routes() {
        for flag in ["--bench-frames","--studio","--cmd","--list-cells","--load","--menu","--cell","--grid"] {
            assert!(!requested(&["engine".into(),"--oblivion-main-menu".into(),flag.into()]));
        }
        assert!(!requested(&["--cell".into(),"AnvilHeinrichOakenHallsHouse".into()]));
        assert!(requested(&["--oblivion-main-menu".into()]));
    }
}
