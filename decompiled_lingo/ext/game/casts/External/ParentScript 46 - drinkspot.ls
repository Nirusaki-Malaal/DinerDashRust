property type, kitchen, drinknode, node, drinks, mouselistener, room_mgr, Tile, destTile, timer, hilite, hilited, checkmarks, numSelects, refreshMS, alphaspeed, alphadir, alphacount, moodPoints

on new me, Args
  type = #drinkspot
  node = Args.node
  node.setVisible(1)
  drinknode = node["drinks"]
  drinknode.setVisible(0)
  room_mgr = Args.room_mgr
  alphaspeed = 10
  alphadir = 1
  alphacount = 0
  moodPoints = getConfigData().drinkMood
  refreshMS = getConfigData().drinkRefreshMS * Args.timeMultiplier
  mouselistener = newMouseListener([#node: node, #target: me, #recursive: 1])
  timer = timedCall(refreshMS, [#target: me, #handler: #refreshDrinks])
  drinks = 0
  hilite = node["hilite"]
  checkmarks = []
  repeat with i = 1 to 3
    checkmarks.add(node["checkmark_" & i])
  end repeat
  me.setHilite(0)
  me.setSelected(0)
  return me
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
  hilite.setVisible(h)
  hilited = h
  if h then
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

on setKitchen me, k
  kitchen = k
end

on doFloAction me, flo
  if not drinks then
    return #nothing_to_do
  end if
  if flo.pickUp(#drinks, [#type: #drinks, #moodPoints: moodPoints]) then
    drinks = 0
    drinknode.setVisible(0)
    me.setHilite(0)
    timer.start()
    return #Ok
  end if
  return #hands_full
end

on refreshDrinks me
  if drinks then
    return 
  end if
  drinks = 1
  me.setHilite(1)
  drinknode.setVisible(1)
end

on destroy me
  unsubscribe(me, #exitFrame)
  if not voidp(mouselistener) then
    mouselistener.destroy()
  end if
  mouselistener = VOID
  if not voidp(timer) then
    timer.destroy()
  end if
  timer = VOID
  hilite = VOID
  checkmarks.deleteAll()
  kitchen = VOID
  Tile = VOID
  drinknode = VOID
  node = VOID
end
