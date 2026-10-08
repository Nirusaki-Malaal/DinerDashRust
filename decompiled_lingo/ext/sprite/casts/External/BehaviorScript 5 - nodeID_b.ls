property nodeID

on getPropertyDescriptionList me
  c = [:]
  c[#comment] = "node ID"
  c[#format] = #string
  c[#default] = EMPTY
  d = [:]
  d[#comment] = "this is a dummy of:"
  d[#format] = #string
  d[#default] = EMPTY
  e = [:]
  e[#comment] = "this is a frame of animation:"
  e[#format] = #string
  e[#default] = EMPTY
  h = [:]
  h[#comment] = "dynamic text?"
  h[#format] = #boolean
  h[#default] = 0
  f = [:]
  f[#comment] = "no sprite"
  f[#format] = #boolean
  f[#default] = 0
  g = [:]
  g[#comment] = "exclude"
  g[#format] = #boolean
  g[#default] = 0
  return [#nodeID: c, #dummyID: d, #animID: e, #dynamic: h, #noSprite: f, #exclude: g]
end
