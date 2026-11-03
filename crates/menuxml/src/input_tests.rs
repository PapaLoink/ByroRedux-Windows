//! Synthetic engineering fixtures for authored menu input and font-driven layout.
use crate::{*,eval::EvalState,parse::{MenuFileSource,parse_document}};
use std::{collections::HashMap,sync::Arc};
struct Empty;
impl MenuFileSource for Empty {fn menu_xml(&self,_:&str)->Option<Vec<u8>>{None}}
fn font()->Font {
    Font {point_size:16.0,texture_name:String::new(),
        glyphs:vec![Glyph{u0:0.0,v0:0.0,u1:1.0,v1:1.0,width:8.0,height:16.0,advance:8.0,x_offset:0.0,y_offset:0.0,inked:true};256],
        atlas:Rgba8{width:1,height:1,pixels:vec![255;4]}}
}
#[test]
fn front_end_hit_rectangles_follow_locus_clipping_and_parent_visibility() {
    let doc=parse_document(r#"<menu name="Menu"><locus>2</locus>
        <rect name="container"><locus>2</locus><x>20</x><y>30</y><width>100</width><height>40</height><clipwindow>2</clipwindow>
        <rect name="button"><id>7</id><target>2</target><alpha>0</alpha><x>5</x><width>120</width><height>20</height></rect>
        </rect></menu>"#,&mut Empty);
    let strings=HashMap::new();let mut overrides=Overrides::new();
    let mut eval=EvalState::new(&doc,ScreenTraits::new(1280.0,960.0),&strings,&overrides);
    let targets=layout::build_layout(&doc,&mut eval).1;
    assert_eq!(targets.len(),1);
    assert_eq!(targets[0].rect,Rect{x:25.0,y:30.0,w:95.0,h:20.0});
    overrides.insert(("container".into(),"visible".into()),Scalar::Num(0.0));
    let mut eval=EvalState::new(&doc,ScreenTraits::new(1280.0,960.0),&strings,&overrides);
    let (draws,targets)=layout::build_layout(&doc,&mut eval);
    assert!(draws.is_empty()&&targets.is_empty());
}
#[test]
fn front_end_button_width_uses_its_own_child_text_and_loaded_font() {
    let doc=parse_document(r#"<menu name="Menu"><locus>2</locus>
        <rect name="button"><id>2</id><target>2</target><locus>2</locus><height>30</height>
          <width><copy src="child(label)" trait="width"/><add>10</add></width>
          <text name="label"><string>Test</string></text>
        </rect></menu>"#,&mut Empty);
    let strings=HashMap::new();let overrides=Overrides::new();let fonts=vec![None,Some(Arc::new(font()))];
    let mut eval=EvalState::new(&doc,ScreenTraits::new(1280.0,960.0),&strings,&overrides).with_fonts(&fonts);
    let targets=layout::build_layout(&doc,&mut eval).1;
    assert_eq!(targets[0].rect.w,42.0);
}

#[test]
fn front_end_non_square_font_metrics_keep_horizontal_and_vertical_axes() {
    let mut fnt=vec![0u8;296+256*56];
    fnt[..4].copy_from_slice(&14.0f32.to_le_bytes());
    let metrics=[0.0f32,0.25,0.125,0.75,0.125,0.25,0.875,0.75,0.875,2.0,6.0,0.0,-1.0,3.0];
    for (i,value) in metrics.into_iter().enumerate(){let at=296+b'A' as usize*56+i*4;fnt[at..at+4].copy_from_slice(&value.to_le_bytes());}
    let mut tex=Vec::new();tex.extend_from_slice(&4u32.to_le_bytes());tex.extend_from_slice(&8u32.to_le_bytes());tex.extend_from_slice(&[255;128]);
    let font=Font::parse(&fnt,&tex).unwrap();let glyph=font.glyph(b'A');
    assert_eq!((glyph.u0,glyph.v0,glyph.u1,glyph.v1),(0.25,0.125,0.75,0.875));
    assert_eq!((glyph.width,glyph.height,glyph.advance),(2.0,6.0,1.0));
    assert_eq!(glyph.y_offset,3.0);
}

#[test]
fn font_spacing_and_baselines_use_independent_metrics_for_both_profiles() {
    // Synthetic FNT/TEX engineering data: no original font bytes.
    // Both MenuXml profiles use the same 56-byte glyph record layout.
    for profile in [MenuProfile::oblivion(), MenuProfile::fallout_nv()] {
        assert!(!profile.font_paths.is_empty());
        let mut fnt=vec![0u8;296+256*56];
        fnt[..4].copy_from_slice(&4.0f32.to_le_bytes());
        for (ch,width,height,left,right,ascent) in [
            (b'A',3.0f32,4.0,1.0,2.0,4.0),
            (b'i',1.0,2.0,0.0,-1.0,2.0),
            (b'g',2.0,4.0,0.0,0.0,2.0),
            (b' ',0.0,0.0,5.0,0.0,0.0),
            (b'_',0.0,0.0,0.0,3.0,0.0),
        ] {
            let values=[0.0,0.0,0.0,1.0,0.0,0.0,1.0,1.0,1.0,width,height,left,right,ascent];
            for (i,v) in values.into_iter().enumerate() {
                let at=296+ch as usize*56+i*4;
                fnt[at..at+4].copy_from_slice(&v.to_le_bytes());
            }
        }
        let mut tex=Vec::new();
        tex.extend_from_slice(&1u32.to_le_bytes());tex.extend_from_slice(&1u32.to_le_bytes());tex.extend_from_slice(&[255;4]);
        let font=Font::parse(&fnt,&tex).unwrap();
        assert_eq!(font.measure_width("A g_"),16.0);
        assert_eq!(font.glyph(b'i').advance,0.0,"a zero advance is valid");
        let mut fb=Framebuffer::new(20,8);
        fb.text_line(&font,"Ag",0.0,0.0,0,raster::BlitStyle{tint:[255.0;3],alpha:255.0,clip:None});
        let alpha=|x:usize,y:usize|fb.pixels[(y*20+x)*4+3];
        assert_eq!(alpha(0,0),0,"left bearing leaves a gap");
        assert_eq!(alpha(1,0),255,"capital starts at baseline minus ascent");
        assert_eq!(alpha(6,0),0,"descender begins lower than the capital");
        assert_eq!(alpha(6,2),255);
        assert_eq!(alpha(6,5),255,"descender extends below baseline");
    }
}
