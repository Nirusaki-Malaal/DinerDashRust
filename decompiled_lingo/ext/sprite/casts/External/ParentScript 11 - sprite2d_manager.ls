property glob, sprites, spriteRange, offscreen
global numcalls, totaltime, avgtime, maxtime

on new me, g
  offscreen = point(-5000, -5000)
  glob = g
  spriteRange = [11, 999]
  sprites = []
  me.makePools()
  return me
end

on makePools me
  m = the milliSeconds
  repeat with i = spriteRange[1] to spriteRange[2]
    s = sprite(i)
    s.puppet = 1
    sprites.add(s)
  end repeat
end

on getSprite me
  if sprites.count = 0 then
    complain("out of sprites")
    return 
  end if
  s = sprites[1]
  s.backColor = 0
  s.foreColor = 255
  s.bgColor = rgb(255, 255, 255)
  s.color = rgb(0, 0, 0)
  s.loc = point(-5000, -5000)
  sprites.deleteAt(1)
  return s
end

on returnSprite me, s
  if s.ilk <> #sprite then
    complain("tried to return something not a sprite")
  end if
  if sprites.getOne(s) <> 0 then
    complain("tried to return a sprite already in pool")
    return 
  end if
  if s.member.name = "bar_bg" then
    nothing()
  end if
  s.scriptInstanceList.deleteAll()
  s.backColor = 0
  s.foreColor = 255
  s.bgColor = rgb(255, 255, 255)
  s.color = rgb(0, 0, 0)
  s.member = member(0)
  s.loc = offscreen
  sprites.add(s)
end

on init me
end

on destroy me
  sprites.deleteAll()
end
