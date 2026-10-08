property sitter, node, home, facenum, oldface, flashtimer, flashstate, flashcount, numflashes, flashtime, color, type, mood, waitgroup_view, mouselistener, number, mouseinside

on new me, Args
  node = Args.node
  node.setVisible(0)
  waitgroup_view = Args[#waitgroup_view]
  number = Args[#number]
  if not voidp(waitgroup_view) then
    mouselistener = newMouseListener([#node: node])
  end if
  facemems = []
  repeat with i = 1 to 4
    facemems.add(member("miniface_" & string(i), "gfx.people"))
  end repeat
  numflashes = 1
  flashtime = 500
  flashcount = 0
  flashstate = 1
  flashtimer = script("timeout").new()
  flashtimer.setCallback([#target: me, #handler: #flash])
  flashtimer.period = flashtime
  home = node.getLoc()[2]
  return me
end

on startFlashing me, ms
  flashcount = 0
  if ms then
    timedCall(ms, [#target: me, #handler: #startFlashing])
  else
    if not voidp(sitter) then
      flashcount = 0
      flashstate = 0
      flashtimer.start()
      me.flash()
    end if
  end if
end

on flash me
  if voidp(sitter) or mouseinside then
    return 
  end if
  if voidp(sitter.sitterGroup) then
    return 
  end if
  flashstate = not flashstate
  if flashstate then
    node.setMember(member("standing_" & sitter.sitterGroup.graphicType & "_4", "gfx.people." & sitter.color))
  else
    me.update(1)
    flashcount = flashcount + 1
    if flashcount >= numflashes then
      me.stopFlashing()
      return 
    end if
  end if
end

on stopFlashing me
  flashtimer.stop()
  flashcount = 0
  me.update(1)
end

on destroy me
  node = VOID
  facenode = VOID
  waitgroup_view = VOID
  if not voidp(mouselistener) then
    mouselistener.destroy()
    mouselistener = VOID
  end if
  if not voidp(sitter) then
    sitter.removeView(me)
  end if
end

on update me, noflash
  if voidp(sitter) then
    return 
  end if
  if voidp(sitter.sitterGroup) then
    return 
  end if
  m = sitter.sitterGroup.mood
  if m = 0 then
    return 
  end if
  if (m <> facenum) or noflash then
    num = max(1, min(integer(m / 3.29999999999999982) + 1, 3))
    cst = "gfx.people." & sitter.color
    t = sitter.sitterGroup.graphicType
    node.setMember(member("standing_" & t & "_" & num, cst))
    facenum = m
  end if
end

on setSitter me, s
  if not voidp(sitter) then
    sitter.removeView(me)
  end if
  flashtimer.stop()
  flashcount = 0
  if voidp(s) then
    node.setVisible(0)
    return 
  end if
  sitter = s
  sitter.addView(me)
  t = sitter.sitterGroup.graphicType
  if t = "1" then
    nothing()
  end if
  facenum = sitter.sitterGroup.mood
  node.setVisible(1)
  me.update(1)
end

on setMouseListening me, ml
  if voidp(mouselistener) then
    return 
  end if
  if ml then
    mouselistener.forwardEvents(me)
  else
    mouselistener.clearCallbacks()
  end if
end

on mouseEnter me
  mouseinside = 1
  if sitter.sitterGroup = VOID then
    me.setSitter(VOID)
    return 
  end if
  node.setMember(member("standing_" & sitter.sitterGroup.graphicType & "_4", "gfx.people." & sitter.color))
  if not waitgroup_view.dragicon.dragging then
    setcursor(#hand_open)
  end if
end

on mouseDown me
  if not voidp(waitgroup_view) then
    waitgroup_view.clickCustomer(number)
  end if
end

on mouseUp me
  if not voidp(waitgroup_view) then
    waitgroup_view.mouseRelease()
  end if
end

on mouseLeave me
  mouseinside = 0
  me.update(1)
  if not waitgroup_view.dragicon.dragging then
    setcursor(#arrow)
  end if
end
