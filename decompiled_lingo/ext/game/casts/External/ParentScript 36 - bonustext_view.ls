property node, kids, showing, home_yreg, speed, height, y, alpha

on new me, n
  node = n
  kids = node.children.duplicate()
  node.setZOffset(1000000)
  home_yreg = node.getYReg()
  speed = 30.0
  height = 30.0
  n.setVisible(0)
  return me
end

on show me, pts
  call(#setText, kids, "+" & string(pts))
  y = 0
  alpha = 100
  node.setVisible(1)
  node.setAlpha(100)
  node.setYReg(home_yreg)
  subscribe(me, #exitFrame)
end

on hide me
  node.setVisible(0)
  unsubscribe(me, #exitFrame)
end

on exitFrame me, ms
  y = y + (ms * 0.001 * speed)
  if y > height then
    me.hide()
    return 
  end if
  node.setYReg(home_yreg + y)
  node.setAlpha((1.0 - (y / height)) * 100)
end

on destroy me
  node = VOID
  kids.deleteAll()
  unsubscribe(me, #exitFrame)
end
