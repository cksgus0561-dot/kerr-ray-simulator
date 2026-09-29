struct Uniforms {
    matrix: mat4x4<f32>,
    viewport: vec4<f32>, // width,height,line pixels,point pixels
    times: vec4<f32>, // t,bin lower,bin upper,propagation mode
    flags: vec4<u32>, // visibility mask, selected ID, detector mode, reserved
};
@group(0) @binding(0) var<uniform> u:Uniforms;
struct Out { @builtin(position) position:vec4<f32>, @location(0) color:vec4<f32>, @location(1) uv:vec2<f32>, @location(2) @interpolate(flat) point:u32 };
fn enabled(kind:u32)->bool{return (u.flags.x & (1u<<kind))!=0u;}
fn hidden()->Out {var o:Out;o.position=vec4<f32>(2.,2.,2.,1.);o.color=vec4<f32>(0.);o.uv=vec2<f32>(0.);o.point=0u;return o;}
fn quad(i:u32)->vec2<f32>{let c=array<vec2<f32>,6>(vec2<f32>(0.,-1.),vec2<f32>(1.,-1.),vec2<f32>(1.,1.),vec2<f32>(0.,-1.),vec2<f32>(1.,1.),vec2<f32>(0.,1.));return c[i];}
fn make_line(a:vec3<f32>,b:vec3<f32>,color:vec4<f32>,kind:u32,id:u32,vertex:u32,point:bool)->Out{
    let p=u.matrix*vec4<f32>(a,1.);let q=u.matrix*vec4<f32>(b,1.);
    if p.w<=0.001||q.w<=0.001{return hidden();}
    var width=u.viewport.z;
    if kind==0u {width=0.7;}
    if kind==2u||kind==5u||kind==7u{width=1.6;}
    var tint=color;if id==u.flags.y && id!=0xffffffffu{tint=vec4<f32>(1.,0.93,0.38,1.);width=width*2.;}
    let c=quad(vertex);var o:Out;var pos=mix(p,q,c.x);o.point=0u;
    if point{
        let scale=select(u.viewport.w,u.viewport.w*1.6,id==u.flags.y);
        pos=p;pos=vec4<f32>(pos.xy+vec2<f32>(c.x*2.-1.,c.y)*scale/u.viewport.xy*pos.w,pos.zw);o.point=1u;
    }else{
        let d=(q.xy/q.w-p.xy/p.w)*u.viewport.xy;
        if length(d)<0.001{return hidden();}
        let normal=normalize(vec2<f32>(-d.y,d.x));
        pos=vec4<f32>(pos.xy+normal*c.y*width/u.viewport.xy*pos.w,pos.zw);
    }
    o.position=pos;o.color=tint;o.uv=vec2<f32>(c.x*2.-1.,c.y);return o;
}
@vertex fn vs_line(@builtin(vertex_index)vertex:u32,@location(0)a:vec4<f32>,@location(1)b:vec4<f32>,@location(2)color:vec4<f32>,@location(3)tag:vec4<u32>)->Out{
    let kind=tag.x;if !enabled(kind){return hidden();}
    var end=b.xyz;
    if kind==8u && u.times.w==1. {
        if a.w>u.times.x{return hidden();}
        let f=clamp((u.times.x-a.w)/max(b.w-a.w,0.000001),0.,1.);end=mix(a.xyz,b.xyz,f);
    }
    if kind==9u{
        if u.flags.z==2u&&f32(tag.z)>=u.times.y{return hidden();}
        if u.flags.z==1u&&(u.times.z<0.||tag.w!=u32(max(u.times.z,0.))){return hidden();}
    }
    return make_line(a.xyz,end,color,kind,tag.y,vertex,distance(a.xyz,b.xyz)<0.000001);
}
@vertex fn vs_tip(@builtin(vertex_index)vertex:u32,@location(0)a:vec4<f32>,@location(1)b:vec4<f32>,@location(2)color:vec4<f32>,@location(3)tag:vec4<u32>)->Out{
    if !enabled(11u)||u.times.w==0.||u.times.x<a.w||u.times.x>=b.w{return hidden();}
    let point=mix(a.xyz,b.xyz,clamp((u.times.x-a.w)/max(b.w-a.w,0.000001),0.,1.));
    return make_line(point,point,vec4<f32>(1.,0.98,0.84,1.),11u,tag.y,vertex,true);
}
@vertex fn vs_surface(@location(0)position:vec3<f32>,@location(1)kind:u32,@location(2)color:vec4<f32>)->Out{
    if !enabled(kind){return hidden();}
    var o:Out;o.position=u.matrix*vec4<f32>(position,1.);o.color=color;o.uv=vec2<f32>(0.);o.point=0u;return o;
}
@fragment fn fs_color(o:Out)->@location(0)vec4<f32>{if o.point==1u&&dot(o.uv,o.uv)>1.{discard;}return o.color;}

@group(1) @binding(0) var overlay:texture_2d<f32>;
@group(1) @binding(1) var texture_sampler:sampler;
struct TextureOut{@builtin(position)position:vec4<f32>,@location(0)uv:vec2<f32>};
@vertex fn vs_overlay(@location(0)position:vec3<f32>,@location(1)uv:vec2<f32>)->TextureOut{
    var o:TextureOut;o.position=u.matrix*vec4<f32>(position,1.);o.uv=uv;return o;
}
@fragment fn fs_overlay(o:TextureOut)->@location(0)vec4<f32>{let c=textureSample(overlay,texture_sampler,o.uv);if c.a==0.{discard;}return c;}
