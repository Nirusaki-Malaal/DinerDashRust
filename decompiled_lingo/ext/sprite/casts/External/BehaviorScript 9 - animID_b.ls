property nodeID

on getPropertyDescriptionList me
  e = [:]
  e[#comment] = "this is a frame of animation:"
  e[#format] = #string
  e[#default] = EMPTY
  a = [:]
  a[#comment] = "regpoint"
  a[#format] = #boolean
  a[#default] = 0
  b = [:]
  b[#comment] = "member"
  b[#format] = #boolean
  b[#default] = 0
  c = [:]
  c[#comment] = "loc"
  c[#format] = #boolean
  c[#default] = 0
  d = [:]
  d[#comment] = "rotation"
  d[#format] = #boolean
  d[#default] = 0
  e = [:]
  e[#comment] = "flip"
  e[#format] = #boolean
  e[#default] = 0
  return [#animID: e, #regPoint: a, #member: b, #loc: c, #rotation: d, #flip: e]
end
