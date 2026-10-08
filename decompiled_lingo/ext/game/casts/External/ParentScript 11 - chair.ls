property Tile, type, sitter, table, views, color, colormems, game_mgr, currentBonus, chairBonus, maxBonus
global glob

on new me, Args
  game_mgr = glob.managers.game
  type = #chair
  table = Args[#table]
  color = Args.data.color
  currentBonus = 1
  cfg = getConfigData()
  maxBonus = cfg.maxChairBonusMultiplier
  chairBonus = cfg.baseChairBonus
  views = []
  return me
end

on setTile me, t
  if Tile = t then
    return 
  end if
  Tile = t
end

on getTile me
  return Tile
end

on setTable me, t
  table = t
  call(#setTable, views, t)
end

on setSitter me, s
  if sitter = s then
    return 
  end if
  sitter = s
end

on checkBonus me
  if voidp(sitter) then
    return 
  end if
  if (color = #grey) or (sitter.color = #none) then
    return 
  end if
  if color <> sitter.color then
    color = sitter.color
    currentBonus = 1
    call(#update, views)
    return 
  end if
  if currentBonus then
    bp = currentBonus * chairBonus
    game_mgr.addPoints(bp)
    call(#showBonus, views, [#score: bp, #multiplier: currentBonus, #color: color])
  end if
  currentBonus = min(currentBonus + 1, maxBonus)
  call(#update, views)
end

on chairBonus me, num
  bp = num * chairBonus
  game_mgr.addPoints(num * chairBonus)
  call(#showChairBonus, views, [#score: bp, #multiplier: num, #color: color])
end

on getNeighborSitters me
  ls = []
  if voidp(Tile) then
    return ls
  end if
  repeat with t in Tile.getNeighbors(#chair)
    s = t.occupant.sitter
    if not voidp(s) then
      ls.add(s)
    end if
  end repeat
  repeat with t in Tile.getDoubleNeighbors(#chair)
    s = t.occupant.sitter
    if not voidp(s) then
      if s.chair.table <> table then
        ls.add(s)
      end if
    end if
  end repeat
  return ls
end

on addView me, vu
  if views.count then
    nothing()
  end if
  if views.getOne(vu) = 0 then
    views.add(vu)
  end if
end

on removeView me, vu
  views.deleteOne(vu)
end

on destroy me
  if not voidp(Tile) then
    Tile.setOccupant(VOID)
    Tile = VOID
  end if
  call(#removeObject, views, me)
  views.deleteAll()
  table = VOID
  sitter = VOID
  game_mgr = VOID
end
