property capacity, chairs, tiles, sitterGroup, tipinfo, room_mgr, game_mgr, dirty, dirtyLevel, maxDirty, states, state, type, shapeinfo, chained, views, tablenum, destTile, neighborTables, mouseinside, bar, startIndex, endIndex
global glob

on new me, Args
  type = #table
  bar = Args[#bar]
  game_mgr = glob.managers.game
  room_mgr = game_mgr.room
  tablenum = Args.tablenum
  neighborTables = []
  maxDirtyLevel = 2
  tiles = []
  views = []
  if voidp(bar) then
    capacity = Args.capacity
    chairs = []
    repeat with i = 1 to capacity
      chairs.add(script("chair", "scripts.game").new([#table: me, #data: Args.data.chairs[i]]))
    end repeat
    if capacity = 4 then
      shapeinfo = [[chairs[1], me, chairs[2]], [chairs[4], me, chairs[3]]]
    else
      if capacity = 2 then
        shapeinfo = [[chairs[1], me, chairs[2]]]
      else
        if capacity = 6 then
          shapeinfo = [[chairs[1], me, chairs[2]], [chairs[6], me, chairs[3]], [chairs[5], me, chairs[4]]]
        end if
      end if
    end if
    v = glob.managers.view.getView(string(capacity) & "top")
    v.setRestaurant(Args[#stage])
    v.setObject(me)
    views.add(v)
  else
    startIndex = Args.startIndex
    endIndex = Args.endIndex
    capacity = endIndex - startIndex + 1
    chairs = Args[#chairs].duplicate()
    call(#setTable, chairs, me)
    shapeinfo = []
    repeat with i = 1 to capacity
      shapeinfo.add([chairs[i], me])
    end repeat
  end if
  sitterChains = []
  return me
end

on addView me, vu
  if views.getOne(vu) = 0 then
    views.add(vu)
  end if
end

on removeView me, vu
  if views.getOne(vu) = 0 then
    return 
  end if
  views.deleteOne(vu)
  vu.removeObject(me)
end

on setSelected me, s
  call(#setSelected, views, s)
end

on mouseover me
  mouseinside = 1
  if the stillDown then
    return 
  end if
  setcursor(#finger_up)
end

on mouseOut me
  if mouseinside then
    setcursor(#arrow)
  end if
  mouseinside = 0
end

on mouseUp me
  if mouseinside then
    setcursor(#finger_up)
  end if
end

on clickOn me
  room_mgr.selectDest(me)
  setcursor(#finger_down)
end

on nextState me
  if not voidp(sitterGroup) then
    sitterGroup.nextState()
  else
    me.clearDishes()
  end if
end

on clearDishes me
  dirty = 0
  game_mgr.checkClosing()
  me.hideDirty()
  call(#clean, views)
  if not voidp(bar) then
    if voidp(sitterGroup) then
      bar.removeFullTable(me)
    end if
  end if
end

on placeFood me, dur
  call(#showFood, views, dur)
  call(#showPlates, views)
  dirty = 1
end

on destroy me
  room_mgr = VOID
  game_mgr = VOID
  sitterGroup = VOID
  tableChain = VOID
  if voidp(bar) then
    call(#destroy, chairs)
  end if
  chairs.deleteAll()
  if not voidp(bar) then
    call(#destroy, views)
    views.deleteAll()
  else
    repeat with v in views.duplicate()
      glob.managers.view.returnView(v)
    end repeat
  end if
end

on getBonus me, Args
  game_mgr.addPoints(Args.score)
  call(#showBonus, views, Args)
end

on placeSitters me, gr, indx
  call(#placeSitters, views, gr, indx)
end

on showMenus me
  call(#showMenus, views)
end

on closeMenus me
  call(#closeMenus, views)
end

on hideMenus me
  call(#hideMenus, views)
end

on hideFood me
  call(#hideFood, views)
end

on showCheck me
  call(#showCheck, views)
end

on hideCheck me
  call(#hideCheck, views)
end

on showDirty me
  call(#showDirty, views)
end

on hideDirty me
  call(#hideDirty, views)
end

on setHilite me, h
  call(#setHilite, views, h)
end

on setFoodPaused me, p
  call(#setFoodPaused, views, p)
end

on showNumber me
  if not voidp(bar) then
    tablenum = bar.getLetter()
  end if
  call(#showNumber, views)
end

on hideNumber me
  if not voidp(bar) then
    bar.returnLetter(tablenum)
  end if
  call(#hideNumber, views)
end

on haveDrink me, pts
  if voidp(sitterGroup) then
    return 
  end if
  sitterGroup.haveDrink(pts)
end

on showBubble me, sn
  if voidp(sn) then
    call(#hideBubble, views)
  else
    call(#showBubble, views, sn)
  end if
end

on setTile me, t
  if voidp(t) then
    call(#removeOccupant, tiles, me)
    tiles.deleteAll()
  else
    if tiles.getOne(t) = 0 then
      tiles.add(t)
    end if
  end if
  call(#align, views)
end

on setDestTile me, t
  destTile = t
end

on getTile me
  if not voidp(destTile) then
    return destTile
  end if
  if tiles.count then
    return tiles[1]
  end if
end

on canHold me, sg
  if voidp(sg) then
    return 
  end if
  return (sg.sitters.count <= capacity) and voidp(sitterGroup) and not dirty
end

on payCheck me
  if voidp(sitterGroup) then
    return 
  end if
  sitterGroup.state.event(#payCheck)
end

on vacate me
  if voidp(sitterGroup) then
    return 
  end if
  sitterGroup.state.event(#leave)
end

on seatGroup me, sgrp, chair, indx
  if not voidp(bar) then
    ct = sgrp.sitters.count
    if bar.full_tables.getOne(me) = 0 then
      st = max(startIndex, min(startIndex + chairs.getPos(chair) - indx, endIndex - ct + 1))
      en = st + ct - 1
      if (st < startIndex) or (en > endIndex) then
        return 
      end if
      t = bar.makeFullTable(st, st + sgrp.sitters.count - 1)
      if voidp(t) then
        return 
      end if
      return t.seatGroup(sgrp, chair, 1)
    end if
  end if
  if (indx < 1) or (indx > capacity) then
    indx = 1
  end if
  if not voidp(sitterGroup) then
    complain("tried to seat a group at an already occupied table")
    return 
  end if
  if not voidp(sgrp.table) then
    complain("tried to seat an already seated group")
    return 
  end if
  indx = ((chairs.getPos(chair) - indx + capacity) mod capacity) + 1
  if indx = 0 then
    chair = chairs[1]
    indx = 1
  end if
  if not voidp(bar) then
    indx = min(indx, capacity - sgrp.sitters.count + 1)
  end if
  sitters = sgrp.sitters
  if sitters.count > chairs.count then
    complain("tried to seat a group a table too small for it")
    return 
  end if
  matchBonus = 0
  repeat with i = 0 to sitters.count - 1
    ch = chairs[((indx + i - 1) mod chairs.count) + 1]
    sit = sitters[i + 1]
    if sit.color = ch.color then
      matchBonus = matchBonus + 1
    end if
    sit.setChair(ch)
    ch.setSitter(sit)
  end repeat
  sitterGroup = sgrp
  sitterGroup.getColorBonus(matchBonus)
  sgrp.setTable(me)
  room_mgr.groupSeated(sgrp, me)
  call(#update, views)
  call(#checkChairBonus, sitters)
  SndSFX("sfx_seat_people")
  call(#setFull, views)
  return me
end

on doFloAction me, flo
  if not voidp(sitterGroup) then
    return sitterGroup.doFloAction(flo)
  else
    if dirty then
      if not voidp(tipinfo) then
        tipinfo = VOID
        call(#hideTip, views)
      end if
      if flo.pickUp(#dishes, me) then
        me.clearDishes()
        return #Ok
      end if
      return #hands_full
    end if
  end if
  return #nothing_to_do
end

on getPossibleActions me
  if not voidp(sitterGroup) then
    return sitterGroup.getPossibleActions()
  end if
  if dirty then
    return [#clean]
  end if
  return []
end

on groupLeave me, sgrp
  if sitterGroup <> sgrp then
    complain("sitter group tried to leave table it wasn't at")
    return 
  end if
  dirtyLevel = dirty
  if dirtyLevel then
    call(#notifyDirty, neighborTables)
  end if
  sitterGroup = VOID
  call(#hideDrinks, views)
  call(#hideMenus, views)
  call(#setSitter, chairs, VOID)
  call(#update, views)
  if not voidp(bar) then
    if not dirty then
      bar.removeFullTable(me)
    end if
  end if
end

on setFoodReady me, b
  me.setHilite(b)
end

on setFoodPickedUp me, b
  if voidp(sitterGroup) then
    return 
  end if
  sitterGroup.state.event(#foodPickedUp, b)
end

on setState me, s
  if state = s then
    return 
  end if
  state = s
end

on setTip me, inf
  tipinfo = inf
  glob.managers.game.scoreTip(tipinfo)
  call(#showTip, views, tipinfo)
end
