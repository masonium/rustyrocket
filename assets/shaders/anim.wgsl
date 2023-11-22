#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals


@group(1) @binding(0)
var base_texture: texture_2d<f32>;
@group(1) @binding(1)
var base_sampler: sampler;

@group(1) @binding(2)
var<uniform> color: vec4f;

@group(1) @binding(3)
var<uniform> scroll_speed: f32;
@group(1) @binding(4)
var<uniform> scroll_direction: f32;
@group(1) @binding(5)
var<uniform> texture_y_mult: f32;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
  var y = in.uv.y;
  if (scroll_direction > 0.0) {
    y = 1.0 - y;
  }
  //let scaled_frag_pos = vec2f(in.uv.x, scroll_direction * (y * 18.0 + scroll_speed * globals.time));
  var scaled_frag_pos = vec2f(in.uv.x, y * texture_y_mult);
  scaled_frag_pos -= vec2f(0.0, scroll_speed * globals.time);
  //let v = vec4<f32>(1.0, 1.0, 1.0, 1.0);

  return textureSample(base_texture, base_sampler, scaled_frag_pos) * color;
}

