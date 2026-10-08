property type, node, foodnode, standnode, mouselistener, room_mgr, waiter, Tile, destTile, hilited, checkmarks, numSelects, foodMembers, snackType, standMembers, wantingGroups, floPayload, bgnode

on new me, Args
  type = #snackspot
  node = Args.node
  bgnode = node.parent
  node.setVisible(1)
  snackType = Args.type
  wantingGroups = []
  foodMembers = [member(string(snackType), "gfx.room"), member(snackType & "_hilite", "gfx.room")]
  foodnode = node["food"]
  foodnode.setMember(foodMembers[1])
  room_mgr = Args.room_mgr
  waiter = room_mgr.waiter
  alphaspeed = 10
  alphadir = 1
  alphacount = 0
  moodPoints = getConfigData().drinkMood
  mouselistener = newMouseListener([#node: node, #target: me, #recursive: 1])
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
  foodnode.setMember(foodMembers[h + 1])
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

on setTile me, t
  Tile = t
  if room_mgr.currentStage <> #finedining then
  end if
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

on addGroup me, g
  if wantingGroups.getOne(g) = 0 then
    wantingGroups.add(g)
    me.updateHilite()
  end if
end

on removeGroup me, g
  if wantingGroups.deleteOne(g) then
    me.updateHilite()
  end if
end

on updateHilite me
  me.setHilite(waiter.getPayload(snackType).count < wantingGroups.count)
end

on doFloAction me, flo
  if flo.pickUp(snackType, VOID) then
    me.updateHilite()
    return #Ok
  end if
  return #hands_full
end

on destroy me
  if not voidp(mouselistener) then
    mouselistener.destroy()
  end if
  voidRefs(me)
end
