property glob, mdls, mouseoverModel, mouseoverListener, mousedownListener, listeners, cursor, event_mgr, initialized

on new me, g
  glob = g
  listeners = [:]
  return me
end

on init me
  initialized = 1
  event_mgr = glob.managers.event
  event_mgr.subscribe(me, #exitFrame)
  grp = glob.managers.node.getGroup("cursor")
  grp.moveToFront()
  cursor = grp.newNode(getNodeData("cursor").children[1])
  cursor.setZOffset(100000000)
  cursor.setLoc(point(-5000, -5000))
  cursor(200)
  subscribe(me, #exitFrame, "cursor")
end

on destroy me
  cursor(-1)
  cursor.destroy()
end

on mouseUp me
  if not initialized then
    return 
  end if
  event_mgr.sendEvent(#mouseRelease)
end

on mouseDown me
  if not initialized then
    return 
  end if
  event_mgr.sendEvent(#mousePress)
end

on mouseWithinNode me, n, recursive
  if not initialized then
    return 
  end if
  if voidp(mdls) then
    return 
  end if
  ret = mdls.getOne(n.pModel)
  if not recursive then
    return ret
  end if
  repeat with i in n.children
    ret = ret or me.mouseWithinNode(i, 1)
  end repeat
  return ret
end

on setcursor me, c
  case c of
    #hand_closed:
      cursor.setMember(member("cursor_grab", "gfx.screens"))
    #hand_open:
      cursor.setMember(member("cursor_open", "gfx.screens"))
    #finger_up:
      cursor.setMember(member("cursor_click_1", "gfx.screens"))
    #finger_down:
      cursor.setMember(member("cursor_click_2", "gfx.screens"))
    #arrow:
      cursor.setMember(member("cursor_arrow", "gfx.screens"))
  end case
end

on exitFrame me
  if not initialized then
    return 
  end if
  cursor.setLoc(the mouseLoc)
end
