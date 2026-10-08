property node, textNode, pmembers, mousehandler, callback, disabled, mousepressed, mouseinside

on new me, Args
  node = Args.node
  textNode = Args[#textNode]
  pmembers = Args[#members]
  if voidp(pmembers) then
    pmembers = [:]
  end if
  m = node.getMember()
  repeat with i in [#normal, #down, #rollover, #toggle]
    if voidp(pmembers[i]) then
      pmembers[i] = m
    end if
  end repeat
  mousehandler = newMouseListener([#node: node, #target: me, #recursive: 1])
  me.setCallback(Args[#callback])
  return me
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
  mousepressed = 1
  node.setMember(pmembers.down)
end

on mouseUp me
  node.setMember(pmembers.rollover)
  if not voidp(callback) then
    call(callback.handler, callback[#target], callback[#Args])
  end if
end

on mouseEnter me
  mouseinside = 1
  if mousepressed then
    node.setMember(pmembers.down)
  else
    node.setMember(pmembers.rollover)
  end if
end

on mouseLeave me
  mouseinside = 0
  mousepressed = 0
  node.setMember(pmembers.normal)
end

on mouseUpOutSide me
  mouseinside = 0
  mousepressed = 0
end

on setDisabled me, d
  disabled = d
  if d then
    mousehandler.clearCallbacks()
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
