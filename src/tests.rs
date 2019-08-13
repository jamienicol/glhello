use gleam::gl;

fn create_texture_data(width: usize, height: usize, pixel: &[u8]) -> Vec<u8> {
    std::iter::repeat(pixel)
        .take(width * height)
        .flatten()
        .copied()
        .collect()
}


/// Test uploading data with a stride which is a multiple of 128 bytes, from a
/// PBO to a non-0th layer of a texture array.
///
/// On Adreno 3xx this will upload to the 0th layer rather than the specified
/// layer.
pub fn test_pbo_to_texture_array_upload(gl: &gl::Gl) {
    println!("Running test_pbo_to_texture_array_upload");

    let black = create_texture_data(32, 32, &[0, 0, 0, 255]);
    let red = create_texture_data(32, 32, &[255, 0, 0, 255]);

    // Create a texture array with 2 layers. Initialize both layers to black.
    let tex = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D_ARRAY, tex);
    gl.tex_storage_3d(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8 as _, 32, 32, 2);
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 0,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &black,
    );
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 1,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &black,
    );

    // Create a PBO with 32x32 red pixels.
    let pbo = gl.gen_buffers(1)[0];
    gl.bind_buffer(gl::PIXEL_UNPACK_BUFFER, pbo);
    gl::buffer_data(
        gl,
        gl::PIXEL_UNPACK_BUFFER,
        &red,
        gl::STREAM_DRAW,
    );

    // Upload from the PBO to layer 1 of the texture.  The important thing is
    // that the width is 32, or a multiple of 32. 32 pixels * 4bpp = 128 bytes.
    // The bug is also reproduced by any width as long as GL_UNPACK_ROW_LENGTH
    // is a multiple of 32.
    gl.tex_sub_image_3d_pbo(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 1,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, 0,
    );

    // Read back a pixel of the 1st layer of the texture. It should be red, but
    // on Adreno 3xx it will be black (and the 0th layer will be red instead).
    let fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, fbo);
    gl.framebuffer_texture_layer(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, tex, 0, 1);

    let pixel = gl.read_pixels(0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE);
    println!("Reading pixel: {:?}", pixel);
    if pixel == [255 as u8, 0, 0, 255] {
        println!("PASS");
    } else {
        println!("FAIL");
    }

    gl.delete_framebuffers(&[fbo]);
    gl.delete_buffers(&[pbo]);
    gl.delete_textures(&[tex]);
}

/// Test blitting from an FBO attached to layer 0 of a texture array to another
/// FBO attached to layer 0 of a second texture array.
///
/// On Adreno 3xx all layers >= the specified layer will be blitted, instead of
/// just the specified layer.
pub fn test_blit_array_only_specified_layer(gl: &gl::Gl) {
    println!("test_blit_array_only_blit_specified_layer");

    let red = create_texture_data(32, 32, &[255, 0, 0, 255]);
    let green = create_texture_data(32, 32, &[0, 255, 0, 255]);
    let black = create_texture_data(32, 32, &[0, 0, 0, 255]);

    // Create a texture array with 2 layers. Fill the 0th layer with red and the
    // 1st layer with green.
    let src_tex = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D_ARRAY, src_tex);
    gl.tex_storage_3d(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8 as _, 32, 32, 2);
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 0,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &red,
    );
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 1,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &green,
    );

    // Create a texture array with 2 layers. Fill both layers with black.
    let dst_tex = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D_ARRAY, dst_tex);
    gl.tex_storage_3d(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8 as _, 32, 32, 2);
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 0,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &black,
    );
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 1,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &black,
    );

    // Blit from the 0th layer of the src texture to the 0th layer of the dest texture.
    let src_fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, src_fbo);
    gl.framebuffer_texture_layer(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, src_tex, 0, 0);

    let dst_fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, dst_fbo);
    gl.framebuffer_texture_layer(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, dst_tex, 0, 0);

    gl.blit_framebuffer(0, 0, 32, 32, 0, 0, 32, 32, gl::COLOR_BUFFER_BIT, gl::NEAREST);

    // Read back a pixel of the 1st layer of the dest texture.  It should be
    // black as we only blitted the 0th layer, but on Adreno 3xx it will be
    // green as the 1st layer was blitted too.
    let read_fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, read_fbo);
    gl.framebuffer_texture_layer(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, dst_tex, 0, 1);

    let pixel = gl.read_pixels(0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE);
    println!("Reading pixel: {:?}", pixel);
    if pixel == [0 as u8, 0, 0, 255] {
        println!("PASS");
    } else {
        println!("FAIL");
    }

    gl.delete_framebuffers(&[src_fbo, dst_fbo, read_fbo]);
    gl.delete_textures(&[src_tex, dst_tex]);
}

/// Test blitting to an FBO attached to a non-0th layer of a texture array.
///
/// On some Adrenos (4xx, 5xx, 6xx) this will blit to the 0th layer instead.
pub fn test_blit_to_texture_array(gl: &gl::Gl) {
    println!("test_blit_to_texture_array");

    let red = create_texture_data(32, 32, &[255, 0, 0, 255]);
    let black = create_texture_data(32, 32, &[0, 0, 0, 255]);

    // Create a texture and fill it with red.
    let src_tex = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D, src_tex);
    gl.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8 as _, 32, 32);
    gl.tex_sub_image_2d(
        gl::TEXTURE_2D, 0,
        0, 0,
        32, 32,
        gl::RGBA, gl::UNSIGNED_BYTE, &red,
    );

    // Create a texture array with 2 layers. Fill both layers with black.
    let dst_tex = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D_ARRAY, dst_tex);
    gl.tex_storage_3d(gl::TEXTURE_2D_ARRAY, 1, gl::RGBA8 as _, 32, 32, 2);
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 0,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &black,
    );
    gl.tex_sub_image_3d(
        gl::TEXTURE_2D_ARRAY, 0,
        0, 0, 1,
        32, 32, 1,
        gl::RGBA, gl::UNSIGNED_BYTE, &black,
    );

    // Blit from the src texture to the 1st layer of the dest texture.
    let src_fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, src_fbo);
    gl.framebuffer_texture_2d(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, src_tex, 0);

    let dst_fbo = gl.gen_framebuffers(1)[0];
    gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, dst_fbo);
    gl.framebuffer_texture_layer(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, dst_tex, 0, 1);

    gl.blit_framebuffer(0, 0, 32, 32, 0, 0, 32, 32, gl::COLOR_BUFFER_BIT, gl::NEAREST);

    // Read back a pixel of the 1st layer of the dest texture. It should be red
    // as we blitted to the 1st layer, but on some Adrenos it will be black and
    // the 0th layer will be red instead.
    gl.bind_framebuffer(gl::READ_FRAMEBUFFER, dst_fbo);
    let pixel = gl.read_pixels(0, 0, 1, 1, gl::RGBA, gl::UNSIGNED_BYTE);
    println!("Reading pixel: {:?}", pixel);
    if pixel == [255 as u8, 0, 0, 255] {
        println!("PASS");
    } else {
        println!("FAIL");
    }

    gl.delete_framebuffers(&[src_fbo, dst_fbo]);
    gl.delete_textures(&[src_tex, dst_tex]);
}
