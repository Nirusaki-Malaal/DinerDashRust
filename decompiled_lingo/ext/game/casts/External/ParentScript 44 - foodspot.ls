property type, kitchen, food, foodnode, node, poof, mouselistener, room_mgr, hilite, checkmarks, numSelects, Tile, destTile, hilited, alphaspeed, alphadir, alphacount, mousetrap

on new me, Args
  type = #foodspot
  room_mgr = Args.room_mgr
  node = Args.node
  foodnode = node["food"]
  mousetrap = node["mousetrap"]
  node.setVisible(0)
  poof = node["poof"]
  alphaspeed = 10
  alphadir = 1
  alphacount = 0
  shapeinfo = [[me]]
  checkmarks = []
  repeat with i = 1 to 3
    checkmarks.add(node["checkmark_" & i])
  end repeat
  me.setHilite(0)
  me.setSelected(0)
  mousetrap.addMouseListener(me)
  return me
end

on mouseEnter me
  setcursor(#finger_up)
  me.setHilite(1)
end

on mouseLeave me
  setcursor(#arrow)
  me.setHilite(1)
end

on mouseDown me
  room_mgr.selectDest(me)
  setcursor(#finger_down)
end

on mouseUp me
  setcursor(#finger_up)
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
  if voidp(food) then
    return #nothing_to_do
  end if
  if flo.pickUp(#food, food) then
    if not voidp(food) then
      food.setFoodPickedUp(1)
    end if
    me.setFood(VOID)
    kitchen.addEmptySpot(me)
    return #Ok
  end if
  return #hands_full
end

on setHilite me, h
end

on setSelected me, s
  if s then
    if not voidp(food) then
      food.setFoodReady(1)
    end if
    numSelects = min(3, numSelects + 1)
    checkmarks[numSelects].setVisible(1)
  else
    if not voidp(food) then
      food.setFoodReady(0)
    end if
    numSelects = 0
    call(#setVisible, checkmarks, 0)
  end if
end

on setFood me, t
  if not voidp(food) then
    was = food.tablenum
  else
    was = "VOID"
  end if
  food = t
  if not voidp(food) then
    m = member("tablenum_" & food.tablenum, "gfx.numbers")
    if m.number <> -1 then
      node["tablenum"].setMember(m)
    else
      nothing()
    end if
    SndSFX("sfx_food_ready_1")
    node.setVisible(1)
    poof.setVisible(1)
    poof.setAnimation("poof", [#loopCount: 1, #callback: [#target: poof, #handler: #setVisible, #Args: 0]])
  else
    me.setHilite(0)
    node.setVisible(0)
  end if
end

on destroy me
  if not voidp(mousetrap) then
    mousetrap.removeMouseListener(me)
  end if
  checkmarks.deleteAll()
  voidRefs(me)
end
