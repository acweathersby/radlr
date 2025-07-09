class TextureAtlasGenerator {
  ctx: OffscreenCanvasRenderingContext2D
  canvas: OffscreenCanvas

  glyph_lu: Map<string, { u: number, v: number, w: number, h: number, r: number, y: number }> = new Map

  glyph_offset: number = 0
  size: number = 34


  constructor() {
    this.canvas = document.createElement("canvas").transferControlToOffscreen();
    let ctx = this.canvas.getContext("2d");

    if (!ctx) {
      throw "Context 2d Not Supported!";
    }

    this.ctx = ctx;
    this.canvas.width = 1024;
    this.canvas.height = 1024;

    this.ctx.fillStyle = 'black';
    this.ctx.fillStyle = 'red';

    this.ctx.font = `${this.size}px Arial`
  }

  getGlyph(char: string) {
    let val = this.glyph_lu.get(char);

    if (val) {
      return val;
    }

    let text = this.ctx.measureText(char)

    let data = {
      u: this.glyph_offset / 1024,
      v: 0 / 1024,
      w: text.width / 1024,
      h: this.size / 1024,
      r: text.width / this.size,
      y: text.actualBoundingBoxDescent
    };

    this.glyph_lu.set(char, data);

    this.ctx.fillText(char, this.glyph_offset, this.size - text.actualBoundingBoxDescent)

    this.glyph_offset += text.width

    return data;
  }
}


export class SyntaxGraphEngine {
  gl: WebGL2RenderingContext
  canvas: HTMLCanvasElement

  nodes: { pos: { x: number, y: number }, name: string, color: number }[]
  connections: number[]

  line_buffer: WebGLBuffer
  glyph_buffer: WebGLBuffer

  text_atlas: WebGLTexture

  node_array_a: Float32Array = new Float32Array(512 * 512 * 3)
  node_array_b: Float32Array = new Float32Array(512 * 512 * 3)
  node_texture_a: WebGLTexture
  node_texture_b: WebGLTexture

  line_program: WebGLProgram
  node_program: WebGLProgram
  text_program: WebGLProgram

  node_count: number = 0
  line_count: number = 0
  glyph_count: number = 0

  offset_x: number = 0
  offset_y: number = -500;
  scale: number = 0.25;
  t: number = 0;
  frame_requested: boolean = false;

  line_vao: WebGLVertexArrayObject
  node_vao: WebGLVertexArrayObject
  text_vao: WebGLVertexArrayObject

  camera_uniform: WebGLUniformLocation

  transfer_buffer: ArrayBuffer = new ArrayBuffer(1024 * 4);

  need_ui_buffer_update: boolean = true

  atlas: TextureAtlasGenerator = new TextureAtlasGenerator()

  /// Establishes a node graph engine within the provided HTMLElement. The
  /// context will always take up the entire space available within the ele argument.
  constructor(par_ele: HTMLDivElement) {
    this.canvas = document.createElement("canvas");
    let gl = this.canvas.getContext("webgl2");

    if (!gl) {
      throw "Webgl 2 Not Supported!";
    }

    par_ele.appendChild(this.canvas);
    this.canvas.classList.add("syntax-engine");

    this.gl = gl;

    this.nodes = []
    this.connections = []

    this.node_program = createProgram(gl, node_draw_vert, node_draw_frag);
    this.line_program = createProgram(gl, line_draw_vert, line_draw_frag);
    this.text_program = createProgram(gl, text_draw_vert, text_draw_frag);

    this.line_buffer = createBuffer(gl);
    this.glyph_buffer = createBuffer(gl);

    this.line_vao = createVAO(gl);
    this.node_vao = createVAO(gl);
    this.text_vao = createVAO(gl);

    this.camera_uniform = createBuffer(gl)

    this.text_atlas = createTexture(gl);

    gl.activeTexture(gl.TEXTURE0);

    gl.bindTexture(gl.TEXTURE_2D, this.text_atlas);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1024, 1024, 0, gl.RGBA, gl.UNSIGNED_BYTE, this.atlas.ctx.getImageData(0, 0, 1024, 1024, { colorSpace: "srgb" }));

    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);

    this.node_texture_a = createTexture(gl);
    this.node_texture_b = createTexture(gl);

    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.node_texture_a);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB32F, 512, 512, 0, gl.RGB, gl.FLOAT, this.node_array_a);


    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);


    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.node_texture_b);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB32F, 512, 512, 0, gl.RGB, gl.FLOAT, this.node_array_b);


    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

    // Create base line shader for drawing circles
    if (gl.getError()) {
      throw "Have gl errors"
    }

    gl.useProgram(this.text_program);

    let texture_location = gl.getUniformLocation(this.text_program, "uSampler");
    gl.uniform1i(texture_location, 0);

    let texture_location2 = gl.getUniformLocation(this.text_program, "uNodes");
    gl.uniform1i(texture_location2, 1);
    texture_location2 = gl.getUniformLocation(this.text_program, "uNodesB");
    gl.uniform1i(texture_location2, 2);

    gl.useProgram(this.line_program);

    texture_location2 = gl.getUniformLocation(this.line_program, "uNodes");
    gl.uniform1i(texture_location2, 1);
    texture_location2 = gl.getUniformLocation(this.line_program, "uNodesB");
    gl.uniform1i(texture_location2, 2);

    gl.useProgram(this.node_program);

    texture_location2 = gl.getUniformLocation(this.node_program, "uNodes");
    gl.uniform1i(texture_location2, 1);
    texture_location2 = gl.getUniformLocation(this.node_program, "uNodesB");
    gl.uniform1i(texture_location2, 2);

    gl.bindBufferBase(gl.UNIFORM_BUFFER, 1, this.camera_uniform);

    let block_index = gl.getUniformBlockIndex(this.node_program, "UIData");
    gl.uniformBlockBinding(this.node_program, block_index, 1);

    block_index = gl.getUniformBlockIndex(this.line_program, "UIData");
    gl.uniformBlockBinding(this.line_program, block_index, 1);

    block_index = gl.getUniformBlockIndex(this.text_program, "UIData");
    gl.uniformBlockBinding(this.text_program, block_index, 1);

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    this.setupGL();
    this.setUiBuffer();


    this.setupInputs()
  }

  private setupInputs() {
    new ResizeObserver(() => {
      this.need_ui_buffer_update = true
      this.draw()
    }).observe(this.canvas)

    let pointer_event_id = 0
    let origin_x = 0
    let origin_y = 0
    let start_x = 0
    let start_y = 0
    let pointer_capture = false

    this.canvas.addEventListener("wheel", e => {
      let { x, y, width, height } = this.canvas.getBoundingClientRect();
      let c_x = x;
      let c_y = y;
      let c_w = width;
      let c_h = height;

      let o_x = e.x - c_x - (c_w / 2);
      let o_y = e.y - c_y - (c_h / 2);

      let diff = -Math.max(Math.min(e.deltaY, 1), -1)
      let old_scale = this.scale
      let new_scale = Math.max(old_scale + (old_scale * 0.02) * diff, 0.1)

      this.offset_x += o_x / new_scale - o_x / old_scale;
      this.offset_y += o_y / new_scale - o_y / old_scale;

      this.scale = new_scale
      this.need_ui_buffer_update = true
      this.draw()
    })

    this.canvas.addEventListener("pointerdown", e => {
      start_x = this.offset_x
      start_y = this.offset_y
      origin_x = e.x
      origin_y = e.y
      pointer_capture = true
      pointer_event_id = e.pointerId
      this.canvas.setPointerCapture(e.pointerId)
    })

    this.canvas.addEventListener("pointermove", e => {

      if (!pointer_capture /* || pointer_event_id != e.pointerId */) return

      this.offset_x = start_x + (e.x - origin_x) / this.scale
      this.offset_y = start_y + (e.y - origin_y) / this.scale
      this.need_ui_buffer_update = true
      this.draw()
    })

    this.canvas.addEventListener("pointerup", e => {
      if (!pointer_capture || pointer_event_id != e.pointerId) return
      document.body.releasePointerCapture(e.pointerId)
      pointer_capture = false

    })
  }

  private setUiBuffer() {
    if (!this.need_ui_buffer_update) return;

    this.need_ui_buffer_update = false;

    let { gl } = this;
    let { width, height } = this.canvas.getBoundingClientRect();

    gl.bindBuffer(gl.UNIFORM_BUFFER, this.camera_uniform);
    gl.bufferData(gl.UNIFORM_BUFFER, new Float32Array([width, height, 0, 0, this.offset_x, -this.offset_y, this.scale, this.t]), gl.DYNAMIC_DRAW);
    gl.bindBuffer(gl.UNIFORM_BUFFER, null);
  }

  addNode(x: number, y: number, name: string, [r, g, b]: [number, number, number] = [255, 0, 0]): number {
    let index = this.nodes.length;
    this.nodes.push({ pos: { x, y }, name, color: (r & 255) << 16 | (g & 255) << 8 | (b & 255) << 0 })
    this.glyph_count += name.length;
    return index
  }

  addText() { }


  flip_nodes() {
    let { gl, node_program, line_program, text_program } = this;
    // Copy the nodes to texture 2 and reset the time value
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, this.node_texture_b);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB32F, 512, 512, 0, gl.RGB, gl.FLOAT, this.node_array_a);
    this.t = 0;
    this.need_ui_buffer_update = true;
  }

  update_node(index: number, x: number, y: number) {
    if (index < this.nodes.length) {
      this.nodes[index].pos.x = x
      this.nodes[index].pos.y = y
    }
  }

  clear_nodes() {
    this.nodes.length = 0;
    this.connections.length = 0;
    this.glyph_count = 0;
    this.line_count = 0;
    this.node_count = 0;
  }

  clear_node_buffers() {
    let { gl, node_program, line_program, text_program } = this;

    let clear_array = this.node_array_a;

    for(let i =0; i < clear_array.length; i++ ){
      clear_array[i] = 0
    }

    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.node_texture_a);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB32F, 512, 512, 0, gl.RGB, gl.FLOAT, clear_array);
    gl.bindTexture(gl.TEXTURE_2D, this.node_texture_b);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB32F, 512, 512, 0, gl.RGB, gl.FLOAT, clear_array);
  }

  add_connection(node_a: number, node_b: number) {
    this.connections.push(node_a, node_b);
  }

  update() {
    this.uploadLines();
    this.uploadNodes();
  }

  draw_internal() {
    this.frame_requested = false;

    if (this.t >= 0.9995) this.t = 1;

    let { gl, node_program, line_program, text_program } = this;

    let { width, height } = this.canvas.getBoundingClientRect();

    if (width == 0 || height == 0) return;

    this.setUiBuffer();

    this.canvas.width = width;
    this.canvas.height = height;

    gl.viewport(0, 0, width, height);

    gl.clearColor(0.0, 0.0, 0.0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.bindVertexArray(this.line_vao);
    gl.useProgram(line_program);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, this.line_count);


    gl.bindVertexArray(this.node_vao);
    gl.useProgram(node_program);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, this.node_count);

    gl.bindVertexArray(this.text_vao);
    gl.useProgram(text_program);
    gl.drawArraysInstanced(gl.TRIANGLES, 0, 6, this.glyph_count);

    // Create base line shader for drawing circles
    let error = 0;
    if (error = gl.getError()) {
      gl.INVALID_ENUM
      gl.INVALID_OPERATION
      console.log({ error });
     // throw "Have gl errors"
    }

    if (this.t < 1) {

      this.t = Math.min(1, this.t + (1 - this.t) / 3);

      this.need_ui_buffer_update = true;

      this.draw()
    }
  }

  draw() {
    if (this.frame_requested) return;
    this.frame_requested = true;
    requestAnimationFrame(() => this.draw_internal())
  }


  private uploadNodes() {

    let node_data_length = 0;
    let glyph_data_length = this.glyph_count * 10;
    let required_buffer_size = (node_data_length + glyph_data_length) << 2;

    let uint = new Uint32Array(this.node_array_a.buffer);

    if (required_buffer_size > this.transfer_buffer.byteLength) {
      this.transfer_buffer = new ArrayBuffer(required_buffer_size);
    } else if (required_buffer_size == 0) {
      return
    }

    let glyphs_f32 = new Float32Array(this.transfer_buffer, node_data_length << 2, glyph_data_length);

    this.node_count = this.nodes.length;

    let glyph_count = 0;

    for (let i = 0, l = this.nodes.length; i < l; i++) {
      let node = this.nodes[i];

      this.node_array_a[i * 3] = node.pos.x
      this.node_array_a[i * 3 + 1] = node.pos.y
      uint[i * 3 + 2] = node.color;

      let input = node.name;
      let stride = 10
      let offset = 0;
      let node_id = i;
      {
        for (let j = 0; j < input.length; j++) {
          let char = input[j];

          let c = j * stride + glyph_count;

          let data = this.atlas.getGlyph(char);

          glyphs_f32[c + 0] = offset
          glyphs_f32[c + 1] = -data.y + 20

          let height = this.atlas.size;

          glyphs_f32[c + 2] = height * data.r
          glyphs_f32[c + 3] = height

          glyphs_f32[c + 4] = data.u
          glyphs_f32[c + 5] = data.v

          glyphs_f32[c + 6] = data.w
          glyphs_f32[c + 7] = data.h

          glyphs_f32[c + 8] = node_id;

          offset += glyphs_f32[c + 2];
        }

        for (let j = 0; j < input.length; j++) {
          let c = j * stride + glyph_count;
          glyphs_f32[c + 0] -= offset / 2
        }

        glyph_count += input.length * stride;
      }
    }

    let { gl, glyph_buffer } = this;

    gl.activeTexture(gl.TEXTURE1);
    gl.bindTexture(gl.TEXTURE_2D, this.node_texture_a);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGB32F, 512, 512, 0, gl.RGB, gl.FLOAT, this.node_array_a);

    gl.bindBuffer(gl.ARRAY_BUFFER, glyph_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, glyphs_f32, gl.DYNAMIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, this.text_atlas);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1024, 1024, 0, gl.RGBA, gl.UNSIGNED_BYTE, this.atlas.ctx.getImageData(0, 0, 1024, 1024, { colorSpace: "srgb" }));

  }

  private uploadLines() {

    let required_buffer_size = this.connections.length * 2 * 4;

    if (required_buffer_size > this.transfer_buffer.byteLength) {
      this.transfer_buffer = new ArrayBuffer(required_buffer_size);
    } else if (required_buffer_size == 0) {
      return
    }

    let line_data = new Float32Array(this.transfer_buffer, 0, required_buffer_size >> 2);

    this.line_count = this.connections.length / 2;

    for (let i = 0, l = this.connections.length; i < l; i += 2) {
      line_data[i * 2 + 0] = this.connections[i];
      line_data[i * 2 + 1] = this.connections[i + 1];
      line_data[i * 2 + 2] = 5;
      line_data[i * 2 + 3] = 5;
    }

    let { gl, line_buffer } = this;
    gl.bindBuffer(gl.ARRAY_BUFFER, line_buffer);
    gl.bufferData(gl.ARRAY_BUFFER, line_data, gl.DYNAMIC_DRAW);
    gl.bindBuffer(gl.ARRAY_BUFFER, null);


  }

  private setupGL() {
    let { gl, node_program, line_program, line_buffer, glyph_buffer, line_vao, text_vao, text_program, node_vao } = this;
    // Create base line shader for drawing circles

    {

      // Glyph data
      gl.bindVertexArray(text_vao);

      gl.bindBuffer(gl.ARRAY_BUFFER, glyph_buffer);

      let stride = 10 * 4;

      var dim_attr = gl.getAttribLocation(text_program, "pos");
      gl.enableVertexAttribArray(dim_attr);
      gl.vertexAttribPointer(dim_attr, 2, gl.FLOAT, false, stride, 0);
      gl.vertexAttribDivisor(dim_attr, 1);

      var pos_attr = gl.getAttribLocation(text_program, "dim");
      gl.enableVertexAttribArray(pos_attr);
      gl.vertexAttribPointer(pos_attr, 2, gl.FLOAT, false, stride, 2 * 4);
      gl.vertexAttribDivisor(pos_attr, 1);

      var uv_attr = gl.getAttribLocation(text_program, "uv");
      gl.enableVertexAttribArray(uv_attr);
      gl.vertexAttribPointer(uv_attr, 2, gl.FLOAT, false, stride, 4 * 4);
      gl.vertexAttribDivisor(uv_attr, 1);

      var uv_size = gl.getAttribLocation(text_program, "uv_size");
      gl.enableVertexAttribArray(uv_size);
      gl.vertexAttribPointer(uv_size, 2, gl.FLOAT, false, stride, 6 * 4);
      gl.vertexAttribDivisor(uv_size, 1);

      var node_index = gl.getAttribLocation(text_program, "pos_index");
      gl.enableVertexAttribArray(node_index);
      gl.vertexAttribPointer(node_index, 1, gl.FLOAT, false, stride, 8 * 4);
      gl.vertexAttribDivisor(node_index, 1);

      gl.bindBuffer(gl.ARRAY_BUFFER, null);

      // Line data
      gl.bindVertexArray(line_vao);

      gl.bindBuffer(gl.ARRAY_BUFFER, line_buffer);

      var p1_attr = gl.getAttribLocation(line_program, "points");
      gl.enableVertexAttribArray(p1_attr);
      gl.vertexAttribPointer(p1_attr, 2, gl.FLOAT, false, 4 * 4, 0);
      gl.vertexAttribDivisor(p1_attr, 1);

      var width_attr = gl.getAttribLocation(line_program, "width");
      gl.enableVertexAttribArray(width_attr);
      gl.vertexAttribPointer(width_attr, 2, gl.FLOAT, false, 4 * 4, 2 * 4);
      gl.vertexAttribDivisor(width_attr, 1);
      gl.bindBuffer(gl.ARRAY_BUFFER, null);

      // Node data
      gl.bindVertexArray(node_vao);
      gl.bindBuffer(gl.ARRAY_BUFFER, null);

    }

    // Create base line shader for drawing circles
    if (gl.getError()) {
      throw "Have gl errors"
    }
  }
}

function createTexture(gl: WebGL2RenderingContext): WebGLTexture {
  let texture = gl.createTexture();
  if (!texture) throw "Could not create Texture";
  return texture;
}

function createVAO(gl: WebGL2RenderingContext): WebGLVertexArrayObject {
  let line_vao = gl.createVertexArray();
  if (!line_vao) throw "Could not create VAO";
  return line_vao;
}

function createBuffer(gl: WebGL2RenderingContext): WebGLBuffer {
  let buffer = gl.createBuffer();
  if (!buffer) { throw "Could not create GL buffer"; }
  return buffer;
}

function createProgram(gl: WebGL2RenderingContext, vert_shader_source: string, frag_shader_source: string) {
  var program = gl.createProgram();

  if (!program) { throw "Could not create shader"; }

  let vert_shader = gl.createShader(gl.VERTEX_SHADER);
  if (!vert_shader) { throw "Could not create shader"; }
  gl.shaderSource(vert_shader, vert_shader_source);
  gl.compileShader(vert_shader);

  let vert_log = gl.getShaderInfoLog(vert_shader);
  if (vert_log) {
    console.error(vert_log);
    throw "Could not compile program: Vert Error";
  }

  let frag_shader = gl.createShader(gl.FRAGMENT_SHADER);
  if (!frag_shader) { throw "Could not create shader"; }
  gl.shaderSource(frag_shader, frag_shader_source);
  gl.compileShader(frag_shader);

  let frag_log = gl.getShaderInfoLog(frag_shader);
  if (frag_log) {
    console.error(frag_log);
    throw "Could not compile program: Frag ERror";
  }

  gl.attachShader(program, vert_shader);
  gl.attachShader(program, frag_shader);



  gl.linkProgram(program);

  let program_log = gl.getProgramInfoLog(program);
  if (program_log) {
    console.error(program_log);
    throw "Could not compile program";
  }

  return program;
}

let ui_block = `

uniform UIData {
  vec2 screen_size;
  vec2 mouse_pos;
  vec2 screen_offset;
  float scale;
  float t;
};
`


let node_block = `

uniform sampler2D uNodes;
uniform sampler2D uNodesB;

struct Node {
  vec2 pos;
  vec3 col;
};

Node get_node_data(float index, sampler2D nodes) {
  ivec2 i = ivec2(int(index) % 512, int(index) / 512);
  vec4 node_data = texelFetch(nodes, i, 0);
  Node node;
  node.pos = node_data.xy;

  uint data = floatBitsToUint(node_data.z);

  node.col = vec3(ivec3(
    data >> 16,
    data >> 8,
    data
  ) & 255 ) / 255.0;
  return node;
}

`

let static_square2 = `
#define right 1.0
#define left -1.0
#define top 1.0
#define bottom -1.0

#define top_left vec2(left, top)
#define top_right vec2(right, top)

#define bottom_left vec2(left, bottom)
#define bottom_right vec2(right, bottom)

vec2[6] positions = vec2[6]( 
  // 0 - 2
  top_left,
  top_right,
  bottom_left,
  // 3 - 5
  top_right,
  bottom_right,
  bottom_left
);


bool[6] color_select = bool[6]( 
  // 0 - 2
  true, // top_left,
  false, // top_right,
  true, // bottom_left,
  // 3 - 5
  false, // top_right,
  false, // bottom_right,
  true // bottom_left
);
`

let line_draw_vert = `#version 300 es

in vec2 points;
in vec2 width;

${static_square2}
${ui_block}
${node_block}

smooth out vec3 node_col;

void main(){

  Node node_a_1 = get_node_data(points.x, uNodes);
  Node node_a_2 = get_node_data(points.x, uNodesB);
  
  Node node_b_1 = get_node_data(points.y, uNodes);
  Node node_b_2 = get_node_data(points.y, uNodesB);

  vec2 point1 = mix(node_a_2.pos, node_a_1.pos, t);
  vec2 point2 = mix(node_b_2.pos, node_b_1.pos, t);

  vec2 static_pos = positions[gl_VertexID] ;

  vec2 diff = point2 - point1 ;

  float theta = atan(diff.y, diff.x);

  float _sin = sin(-theta);
  float _cos = cos(-theta);

  mat2 rot =  mat2( _cos, -_sin, _sin, _cos);

  vec2 scaled =  static_pos * vec2(1.0,1.0/ scale) * vec2(length(diff) * 0.5, width.x * 0.5) ;

  vec2 rot_pos = rot * scaled;

  vec2 offset = (point1 + diff * 0.5);

  gl_Position = vec4((rot_pos + offset + screen_offset * 2.0) / screen_size * scale, 0.0, 1);

  if( color_select[gl_VertexID] ) {
    node_col = node_a_1.col;
  } else {
    node_col = node_b_1.col;
  }
}
`;

let line_draw_frag = `#version 300 es

precision highp float;
smooth in vec3 node_col;
out vec4 color;
void main(){
  color = vec4(mix(node_col, vec3(0), 0.2), 1);
}
`

let node_draw_vert = `#version 300 es

${static_square2}
${ui_block}
${node_block}

out vec3 base_color; 
flat out vec2 center_point;
smooth out vec2 actual_point; 

float diameter = 40.0;

void main(){

  Node node_a_1 = get_node_data(float(gl_InstanceID), uNodes);
  Node node_a_2 = get_node_data(float(gl_InstanceID), uNodesB);
  
  vec2 node_pos = mix(node_a_2.pos, node_a_1.pos, t);
\
  vec2 static_pos = positions[gl_VertexID];
 
  actual_point = static_pos;
  center_point = vec2(0, 0);

  vec2 adjusted_pos = ((static_pos) * diameter + node_pos + screen_offset * 2.0) / screen_size * scale;

  gl_Position = vec4(adjusted_pos, 0.0, 1);
  
  base_color = node_a_1.col;
}
`

let node_draw_frag = `#version 300 es

precision highp float;


in vec3 base_color;
flat in vec2 center_point; 
smooth in vec2 actual_point; 

out vec4 color;

void main(){
  float diff = dFdx(actual_point.x);

  float val = length(actual_point - center_point);

  val = smoothstep(0.97 - (0.01 * (diff)), 1.0, val);

  if(val > 1.0)
    discard;

  color = vec4(base_color, 1.0 -val);
}
`;



let text_draw_vert = `#version 300 es

precision highp float;

${static_square2}
${ui_block}
${node_block}

smooth out vec2 UV; 

in float pos_index;
in vec2 pos;
in vec2 dim;
in vec2 uv;
in vec2 uv_size;

void main(){
  Node nodeA = get_node_data(pos_index, uNodes);
  Node nodeB = get_node_data(pos_index, uNodesB);

  vec2 node_pos = mix(nodeB.pos, nodeA.pos, t);

  vec2 static_pos = positions[gl_VertexID];
  gl_Position = vec4(((static_pos + vec2(1)) * 0.5 * vec2(1, -1) * dim + pos ) / screen_size + (screen_offset * 2.0 + node_pos) / screen_size * scale , 0.0, 1.0);
  UV = ((static_pos / 2.0) + vec2(0.5)) * uv_size + uv ;
}`;

let text_draw_frag = `#version 300 es

precision highp float;

smooth in vec2 UV; 

uniform sampler2D uSampler;

out vec4 color;

void main(){

  vec4 texture = texture(uSampler, UV);

  color = vec4(texture.r, 1.0, 1.0, texture.a);
}
`