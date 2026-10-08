property paused, callbacks, nodeList, mouseover, mousepressed
global glob

on new me, Args
  callbacks = [#mouseUp: VOID, #mouseUpOutSide: VOID, #mouseDown: VOID, #mouseEnter: VOID, #mouseLeave: VOID]
  nodeList = []
  if not voidp(Args) then
    if not voidp(Args[#node]) then
      me.addNode(Args.node, Args[#recursive])
    end if
    if not voidp(Args[#target]) then
      me.forwardEvents(Args.target)
    end if
  end if
  return me
end

on destroy me
  me.clearCallbacks()
  me.removeAllNodes()
  mgr = VOID
end

on addNode me, n, recursive
  if voidp(n) then
    return 
  end if
  if nodeList.getOne(n) = 0 then
    nodeList.add(n)
  end if
  n.addMouseListener(me)
  if recursive and (n.type = #Branch) then
    repeat with i in n.children
      me.addNode(i, 1)
    end repeat
  end if
end

on Refresh me
  repeat with i in nodeList
    me.addNode(i)
  end repeat
end

on removeNode me, n, recursive
  nodeList.deleteOne(n)
  n.removeMouseListener(me)
  if not recursive then
    return 
  end if
  repeat with i in n.children
    me.removeNode(i, 1)
  end repeat
end

on removeAllNodes me
  call(#removeMouseListener, nodeList, me)
  nodeList.deleteAll()
end

on forwardEvents me, obj
  me.clearCallbacks()
  h = obj.handlers()
  repeat with i in h
    if not voidp(callbacks.findPos(i)) then
      me.setCallback(i, [#target: obj, #handler: i])
    end if
  end repeat
end

on clearCallbacks me
  repeat with i = 1 to callbacks.count
    callbacks[i] = VOID
  end repeat
end

on setCallback me, sym, cb
  if not voidp(callbacks.findPos(sym)) then
    callbacks[sym] = cb
  end if
end

on mouseUp me
  if paused or not mousepressed then
    return 
  end if
  mousepressed = 0
  c = callbacks.mouseUp
  if voidp(c) then
    return 
  end if
  call(c.handler, c.target, c[#Args])
end

on mouseUpOutSide me
  if paused or not mousepressed then
    return 
  end if
  mousepressed = 0
  mouseover = 0
  c = callbacks.mouseUpOutSide
  if voidp(c) then
    return 
  end if
  call(c.handler, c.target, c[#Args])
end

on mouseDown me
  if paused or mousepressed then
    return 
  end if
  mousepressed = 1
  c = callbacks.mouseDown
  if voidp(c) then
    return 
  end if
  call(c.handler, c.target, c[#Args])
end

on mouseEnter me
  if paused or mouseover then
    return 
  end if
  mouseover = 1
  c = callbacks.mouseEnter
  if voidp(c) then
    return 
  end if
  call(c.handler, c.target, c[#Args])
end

on mouseLeave me
  if paused or not mouseover then
    return 
  end if
  mouseover = 0
  c = callbacks.mouseLeave
  if voidp(c) then
    return 
  end if
  call(c.handler, c.target, c[#Args])
end
