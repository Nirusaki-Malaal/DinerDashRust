property home, node, minY, speed, ploc, moving
global glob

on new me, n
  node = n
  home = node.getLoc()
  ploc = home.duplicate()
  minY = -300
  return me
end

on changeSpeed me
  speed = point((random(20) / 5.0) - 2, -3.0 - (random(10) / 5.0))
end

on setVisible me, v
  node.setVisible(v)
end

on fire me
  timedCall(random(2000), [#target: me, #handler: #start])
end

on start me
  if voidp(node) then
    return 
  end if
  ploc[1] = home[1]
  ploc[2] = home[2]
  me.changeSpeed()
  moving = 1
  subscribe(me, #exitFrame)
end

on exitFrame me
  if not moving then
    return 
  end if
  if random(5) = 1 then
    me.changeSpeed()
  end if
  if ploc[2] < minY then
    me.start()
  end if
  ploc = ploc + speed
  node.setRotation(node.getRotation() + 20)
  node.setLoc(ploc)
end

on stop me
  unsubscribe(me, #exitFrame)
  moving = 0
  node.setLoc(home)
end

on destroy me
  unsubscribe(me, #exitFrame)
  moving = 0
  node = VOID
end
