#version 330 core

in vec2 v_uv;

out vec4 frag_color;

uniform vec2 u_resolution;
uniform vec2 u_player_pos;
uniform vec2 u_player_dir;

uniform sampler2D u_map;
uniform vec2 u_map_size;

uniform sampler2D u_wall_tex;
uniform sampler2D u_floor_tex;
uniform sampler2D u_ceiling_tex;

const int MAX_STEPS = 64;
const float FOG_DISTANCE = 8.0;

void main() {
    vec2 pos = u_player_pos;
    vec2 dir = u_player_dir;

    float camera_x = 2.0 * gl_FragCoord.x / u_resolution.x - 1.0;
    vec2 plane = vec2(-dir.y, dir.x) * 0.66;
    vec2 ray_dir = dir + plane * camera_x;

    int map_x = int(pos.x);
    int map_y = int(pos.y);

    vec2 delta_dist = abs(vec2(1.0 / ray_dir.x, 1.0 / ray_dir.y));

    int step_x;
    int step_y;
    float side_dist_x;
    float side_dist_y;

    if (ray_dir.x < 0.0) {
        step_x = -1;
        side_dist_x = (pos.x - float(map_x)) * delta_dist.x;
    } else {
        step_x = 1;
        side_dist_x = (float(map_x) + 1.0 - pos.x) * delta_dist.x;
    }

    if (ray_dir.y < 0.0) {
        step_y = -1;
        side_dist_y = (pos.y - float(map_y)) * delta_dist.y;
    } else {
        step_y = 1;
        side_dist_y = (float(map_y) + 1.0 - pos.y) * delta_dist.y;
    }

    int hit = 0;
    int side = 0;

    for (int i = 0; i < MAX_STEPS; i++) {
        if (side_dist_x < side_dist_y) {
            side_dist_x += delta_dist.x;
            map_x += step_x;
            side = 0;
        } else {
            side_dist_y += delta_dist.y;
            map_y += step_y;
            side = 1;
        }

        vec2 map_uv = (vec2(float(map_x), float(map_y)) + 0.5) / u_map_size;
        if (texture(u_map, map_uv).r > 0.5) {
            hit = 1;
            break;
        }
    }

    if (hit == 0) {
        frag_color = vec4(0.05, 0.05, 0.08, 1.0);
        return;
    }

    float perp_wall_dist;
    if (side == 0) {
        perp_wall_dist = (float(map_x) - pos.x + (1.0 - float(step_x)) * 0.5) / ray_dir.x;
    } else {
        perp_wall_dist = (float(map_y) - pos.y + (1.0 - float(step_y)) * 0.5) / ray_dir.y;
    }
    perp_wall_dist = abs(perp_wall_dist);

    float wall_x;
    if (side == 0) {
        wall_x = pos.y + perp_wall_dist * ray_dir.y;
    } else {
        wall_x = pos.x + perp_wall_dist * ray_dir.x;
    }
    wall_x -= floor(wall_x);

    float line_height = u_resolution.y / perp_wall_dist;
    float draw_start = -line_height * 0.5 + u_resolution.y * 0.5;
    float draw_end = line_height * 0.5 + u_resolution.y * 0.5;

    float y = gl_FragCoord.y;

    if (y >= draw_start && y <= draw_end) {
        float tex_y = (y - draw_start) / line_height;
        vec3 wall_color = texture(u_wall_tex, vec2(wall_x, tex_y)).rgb;
        float shade = side == 1 ? 0.75 : 1.0;
        float fog = clamp(1.0 - perp_wall_dist / FOG_DISTANCE, 0.25, 1.0);
        frag_color = vec4(wall_color * shade * fog, 1.0);
        return;
    }

    if (y > draw_end) {
        float p = y - u_resolution.y * 0.5;
        float row_dist = (0.5 * u_resolution.y) / p;
        vec2 floor_world = pos + row_dist * ray_dir;
        vec2 floor_uv = fract(floor_world);
        vec3 floor_color = texture(u_floor_tex, floor_uv).rgb;
        float fog = clamp(1.0 - row_dist / FOG_DISTANCE, 0.25, 1.0);
        frag_color = vec4(floor_color * fog, 1.0);
        return;
    }

    float p_ceil = u_resolution.y * 0.5 - y;
    float row_dist_ceil = (0.5 * u_resolution.y) / p_ceil;
    vec2 ceil_world = pos + row_dist_ceil * ray_dir;
    vec2 ceil_uv = fract(ceil_world);
    vec3 ceil_color = texture(u_ceiling_tex, ceil_uv).rgb;
    float fog_ceil = clamp(1.0 - row_dist_ceil / FOG_DISTANCE, 0.25, 1.0);
    frag_color = vec4(ceil_color * 0.85 * fog_ceil, 1.0);
}
