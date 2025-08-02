#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0) var<uniform> light_color: vec4<f32>;
@group(2) @binding(1) var<uniform> dark_color: vec4<f32>;
@group(2) @binding(2) var<uniform> special_color: vec4<f32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let scale = 11.0;

    let scaled_uv = uv * scale;

    let grid_x = floor(scaled_uv.x);
    let grid_y = floor(scaled_uv.y);

    let checkerboard_color = mix(light_color, dark_color, f32(u32(grid_x + grid_y) % 2u));
    
    let is_corner_x = (grid_x == 0.0) || (grid_x == 10.0);
    let is_corner_y = (grid_y == 0.0) || (grid_y == 10.0);
    let is_center_x = (grid_x == 5.0);
    let is_center_y = (grid_y == 5.0);
    
    let is_special_tile = (is_corner_x && is_corner_y) || (is_center_x && is_center_y);
    
    let final_color = select(checkerboard_color, special_color, is_special_tile);

    return final_color;
}
