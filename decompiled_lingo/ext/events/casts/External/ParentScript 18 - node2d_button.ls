property node, textNode, pmembers, mousehandler, callback, disabled, mousepressed, mouseinside, sounds, nosound

on new me, Args
  node = Args.node
  textNode = Args[#textNode]
  if voidp(Args[#members]) then
    pmembers = [:]
  else
    pmembers = Args[#members].duplicate()
  end if
  m = node.getMember()
  repeat with i in [#normal, #down, #rollover, #toggle, #disabled]
    if voidp(pmembers[i]) then
      pmembers[i] = m
    end if
  end repeat
  mousehandler = script("node2d_mouselistener").new([#node: node, #target: me, #recursive: 1])
  callback = Args[#callback]
  nosound = Args[#nosound]
  if not nosound then
    sounds = Args[#sounds]
    if voidp(sounds) then
      sounds = [:]
    end if
    if voidp(sounds[#rollover]) then
      sounds[#rollover] = "sfx_generic_button_rollover"
    end if
    if voidp(sounds[#press]) then
      sounds[#press] = "sfx_generic_button_yes"
    end if
  end if
  return me
end

on setMembers me, mems
  if not ilk(mems, #propList) then
    return 
  end if
  dm = pmembers.normal
  repeat with i in [#normal, #down, #rollover, #toggle, #disabled]
    m = mems[i]
    if voidp(m) then
      m = dm
    end if
    pmembers[i] = m
  end repeat
  if disabled then
    node.setMember(pmembers[#disabled])
  else
    if mousepressed then
      node.setMember(pmembers[#down])
    else
      if mouseinside then
        node.setMember(pmembers[#rollover])
      end if
    end if
  end if
end

on setText me, t
  if voidp(textNode) then
    return 
  end if
  textNode.setText(t)
end

on setVisible me, v
  node.setVisible(v)
  if not voidp(textNode) then
    textNode.setVisible(v)
  end if
end

on setLoc me, p
  node.setLoc(p)
end

on setCallback me, cb
  if callback = cb then
    return 
  end if
  callback = cb
  me.setDisabled(voidp(cb))
end

on mouseDown me
  setcursor(#finger_down)
  mousepressed = 1
  node.setMember(pmembers.down)
end

on mouseUp me
  node.setMember(pmembers.rollover)
  setcursor(#finger_up)
  if not nosound then
    SndSFX(sounds[#press])
  end if
  if not voidp(callback) then
    call(callback.handler, callback[#target], callback[#Args], me)
  end if
end

on mouseEnter me
  mouseinside = 1
  if mousepressed then
    node.setMember(pmembers.down)
    setcursor(#finger_down)
  else
    if not nosound then
      SndSFX(sounds[#rollover])
    end if
    node.setMember(pmembers.rollover)
    setcursor(#finger_up)
  end if
end

on mouseLeave me
  mouseinside = 0
  mousepressed = 0
  node.setMember(pmembers.normal)
  setcursor(#arrow)
end

on mouseUpOutSide me
  mouseinside = 0
  mousepressed = 0
end

on setDisabled me, d
  disabled = d
  if d then
    mousehandler.clearCallbacks()
    mouseinside = 0
    mousepressed = 0
    m = pmembers[#disabled]
    if voidp(m) then
      node.setAlpha(50)
    else
      node.setMember(m)
    end if
  else
    mousehandler.forwardEvents(me)
    node.setAlpha(100)
    if mouseinside then
      node.setMember(pmembers.rollover)
    else
      node.setMember(pmembers.normal)
    end if
  end if
end

on destroy me
  mousehandler.destroy()
  mousehandler = VOID
  textNode = VOID
  node = VOID
  if not voidp(callback) then
    callback.deleteAll()
  end if
end
