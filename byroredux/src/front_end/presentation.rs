//! Compose original movie pixels and MenuXml through the existing overlay.
use super::*;
use byroredux_menuxml::{Framebuffer, Rect, raster::BlitStyle};
use byroredux_renderer::{VulkanContext, vulkan::context::FrameInputs};

fn blit(frame:&mut Framebuffer, image:&Rgba8, rect:Rect) {
    frame.blit(image,rect,(0.0,0.0),-1.0,false,BlitStyle{tint:[255.0;3],alpha:255.0,clip:None});
}
impl FrontEnd {
    pub(super) fn tick(&mut self,world:&byroredux_core::ecs::World,ctx:&mut VulkanContext)->Result<()> {
        {
            let mut audio=world.resource_mut::<AudioWorld>();
            if self.movie.is_none() {self.open_phase(&mut audio)?;}
            let (frame,done)=self.movie.as_mut().unwrap().tick(&mut audio)?;
            if let Some(frame)=frame {self.video=Some(frame);}
            if done {self.advance(&mut audio)?;}
        }
        let extent=ctx.frame_extents.output;
        let (width,height)=(extent.width,extent.height);
        if width==0||height==0{return Ok(());}
        if width>4096||height>4096 {bail!("front-end output exceeds 4096 pixel bound");}
        if self.size!=(width,height) {
            self.menu.set_screen(ScreenTraits::new(960.0*width as f32/height as f32,960.0));
            self.size=(width,height);self.dirty=true;
        }
        if self.dirty && self.phase==Phase::Menu {
            self.targets=self.menu.hit_targets();
            let (w,h)=self.menu.frame_size();
            self.menu_image=Some(Rgba8 {width:w,height:h,pixels:self.menu.render_frame(&self.assets).to_vec()});
            self.dirty=false;
            log::debug!("front-end: authored targets {:?}",self.targets.iter().map(|t|(&t.name,t.rect)).collect::<Vec<_>>());
        }
        let options_image=if let Some(settings)=self.settings.as_mut() {
            Some(settings.frame(&self.assets,self.menu.screen(),&world.resource::<byroredux_core::settings::SettingsRegistry>())?)
        }else{None};
        let mut frame=Framebuffer::new(width,height);frame.clear([0,0,0,255]);
        if let Some(image)=&self.video {
            let scale=(width as f32/image.width as f32).min(height as f32/image.height as f32);
            let w=image.width as f32*scale;let h=image.height as f32*scale;
            blit(&mut frame,image,Rect{x:(width as f32-w)/2.0,y:(height as f32-h)/2.0,w,h});
        }
        if self.phase==Phase::Menu {
            if let Some(image)=options_image.as_ref().or(self.menu_image.as_ref()) {blit(&mut frame,image,Rect{x:0.0,y:0.0,w:width as f32,h:height as f32});}
        }
        let upload=byroredux_renderer::upload_ctx!(ctx);
        if let Some(handle)=self.texture {
            ctx.texture_registry.update_rgba(upload,handle,width,height,&frame.pixels)?;
        } else {
            self.texture=Some(ctx.texture_registry.register_rgba(upload,width,height,&frame.pixels)?);
        }
        Ok(())
    }

    pub(super) fn draw(&self,ctx:&mut VulkanContext)->Result<()> {
        let identity=byroredux_core::math::Mat4::IDENTITY.to_cols_array();
        let sky=byroredux_renderer::SkyParams::default();
        let offsets=Default::default();let dirty=Default::default();
        if ctx.draw_frame(FrameInputs {
            clear_color:[0.0,0.0,0.0,1.0],view_proj:&identity,draw_commands:&[],lights:&[],light_ids:&[],fog_volumes:&[],
            bone_world:&[],skin_offsets:&offsets,bind_inverse_pending_uploads:&[],materials:&[],has_effect_soft_material:false,
            camera_pos:[0.0;3],render_origin:[0.0;3],ambient_color:[0.0;3],fog_color:[0.0;3],fog_near:0.0,fog_far:1000.0,
            fog_extinction_per_meter:0.0,fog_single_scatter_albedo:0.0,fog_coverage:0.0,fog_scale_height_meters:0.0,
            fog_clip:0.0,fog_power:0.0,fog_height_reference:0.0,wind_params:[0.0;4],wind_gust:[0.0;4],
            ui_texture_handle:self.texture,sky_params:&sky,dof:Default::default(),frame_time_delta_ms:16.667,
            timings:None,water_commands:&[],underwater:Default::default(),image_space_modifier:Default::default(),pose_dirty:&dirty,
        })? {let size=ctx.frame_extents.output;ctx.recreate_swapchain([size.width,size.height])?;}
        Ok(())
    }
}
