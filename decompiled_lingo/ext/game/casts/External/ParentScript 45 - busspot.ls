property Tile, destTile, type, node, hilite, checkmarks, numSelects, kitchen, room_mgr, mouselistener, hilited, alphaspeed, alphadir, alphacount, dropoffTypes, fullcount, timer

on new me, Args
  type = #busspot
  node = Args.node
  node.setVisible(1)
  hilite = node["hilite"]
  checkmarks = []
  repeat with i = 1 to 3
    checkmarks.add(node["checkmark_" & i])
  end repeat
  dropoffTypes = [#food, #crab, #somosa, #bread, #papadom]
  alphaspeed = 10
  alphadir = 1
  alphacount = 0
  me.setHilite(0)
  me.setSelected(0)
  room_mgr = Args.room_mgr
  mouselistener = newMouseListener([#target: me, #node: node, #recursive: 1])
  subscribe(me, #exitFrame)
  fullcount = 0
  me.updateFull()
  return me
end

on setKitchen me, k
  kitchen = k
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

on doFloAction me, flo
  result = #nothing_to_do
  if flo.hasPayloadType(#dishes) then
    flo.dropOffAll(#dishes)
    result = #Ok
  end if
  if flo.hasPayloadType(#drinks) then
    flo.dropOffAll(#drinks)
    result = #Ok
  end if
  repeat with typ in dropoffTypes
    if flo.hasPayloadType(typ) then
      if typ = #food then
        fd = flo.getPayload(typ)
        call(#setFoodPickedUp, fd, 0)
        call(#setFoodReady, fd, 0)
        repeat with i in fd
          kitchen.returnFood(i)
        end repeat
      end if
      flo.dropOffAll(typ)
      call(#updateHilite, room_mgr.snackStations)
      result = #Ok
    end if
  end repeat
  if result = #Ok then
    fullcount = fullcount + 1
    me.updateFull()
  end if
  return result
end

on destroyTimer me
  if not voidp(timer) then
    timer.destroy()
    timer = VOID
  end if
end

on updateFull me
  node.setAnimation("full_" & max(0, min(fullcount, 2)))
  if fullcount >= 2 then
    fullcount = 0
    me.destroyTimer()
    timer = timedCall(1500, [#target: me, #handler: #updateFull])
  end if
end

on setHilite me, h
  hilite.setVisible(h)
  hilited = h
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

on destroy me
  me.destroyTimer()
  unsubscribe(me, #exitFrame)
  if not voidp(mouselistener) then
    mouselistener.destroy()
    mouselistener = VOID
  end if
  checkmarks.deleteAll()
  hilite = VOID
  node = VOID
  kitchen = VOID
  room_mgr = VOID
  Tile = VOID
end

on setTile me, t
  Tile = t
end

on setDestTile me, t
  destTile = t
end

on getTile me
  if not voidp(destTile) then
    return destTile
  end if
  return Tile
end
