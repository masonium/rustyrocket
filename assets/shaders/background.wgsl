#import bevy_sprite::mesh2d_types         Mesh2d
#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings globals, view
#import bevy_shader_utils::simplex_noise_2d simplex_noise_2d

@group(1) @binding(0)
var<uniform> c1: vec4f;
@group(1) @binding(1)
var<uniform> c2: vec4f;
@group(1) @binding(2)
var<uniform> time: f32;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
  // var y = in.uv.y;
  // if (scroll_direction > 0.0) {
  //   y = 1.0 - y;
  // }
  let ar = view.projection[1][1] / view.projection[0][0];

  // var scaled_frag_pos = vec2f(in.uv.x, y * 18.0);
  // scaled_frag_pos -= vec2f(0.0, scroll_speed * globals.time);
  let grid = 80.0;
  let uv = vec2f((in.uv.x + time * 0.1) * ar, in.uv.y);
  var g = floor(uv * grid) / grid;

  let v = simplex_noise_2d(g) * 0.1;
  let vg = floor(v * grid) / grid;

  let w = round(fract((g.x + g.y + vg) * 2.0));

  return (1.0 - w) * c1 + w * c2;
}
