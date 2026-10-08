property kitchen, Tile, destTile, type, node, hilite, checkmarks, numSelects, room_mgr, mouselistener, hilited, alphaspeed, alphadir, alphacount

on new me, Args
  type = #ticketspot
  node = Args.node
  node.setVisible(1)
  hilite = node["hilite"]
  checkmarks = []
  alphaspeed = 10
  alphadir = 1
  alphacount = 0
  repeat with i = 1 to 3
    checkmarks.add(node["checkmark_" & i])
  end repeat
  me.setHilite(0)
  me.setSelected(0)
  room_mgr = Args.room_mgr
  mouselistener = newMouseListener([#target: me, #node: node, #recursive: 1])
  subscribe(me, #exitFrame)
  return me
end

on exitFrame me
  if not hilited then
    return 
  end if
  alphacount = alphacount + (alphadir * alphaspeed)
  if alphacount < 0 then
    alphadir = 1
  else
    if alphacount > 100 then
      alphadir = -1
    end if
  end if
  hilite.setAlpha(max(0, min(alphacount, 100)))
end

on mouseEnter me
  setcursor(#finger_up)
end

on mouseLeave me
  setcursor(#arrow)
end

on mouseDown me
  room_mgr.selectDest(me)
  setcursor(#finger_down)
end

on mouseUp me
  setcursor(#finger_up)
end

on setHilite me, h
  node.setAnimation("start")
  hilite.setVisible(h)
  hilited = h
  if hilited then
    subscribe(me, #exitFrame)
  else
    unsubscribe(me, #exitFrame)
  end if
end

on setSelected me, s
  if s then
    numSelects = min(3, numSelects + 1)
    checkmarks[numSelects].setVisible(1)
  else
    numSelects = 0
    call(#setVisible, checkmarks, 0)
  end if
end

on doFloAction me, flo
  if not flo.hasPayloadType(#ticket) then
    return #nothing_to_do
  end if
  repeat with i in flo.getPayload(#ticket)
    kitchen.addTicket(i)
    flo.dropOff(#ticket, i)
    node.setAnimation("turn", [#loopCount: 1])
  end repeat
  return #Ok
end

on setKitchen me, k
  kitchen = k
end

on destroy me
  unsubscribe(me, #exitFrame)
  if not voidp(mouselistener) then
    mouselistener.destroy()
    mouselistener = VOID
  end if
  checkmarks.deleteAll()
  hilite = VOID
  node = VOID
  room_mgr = VOID
  kitchen = VOID
  Tile = VOID
end

on setTile me, t
  Tile = t
end

on getTile me
  if not voidp(destTile) then
    return destTile
  end if
  return Tile
end

on setDestTile me, t
  destTile = t
end
