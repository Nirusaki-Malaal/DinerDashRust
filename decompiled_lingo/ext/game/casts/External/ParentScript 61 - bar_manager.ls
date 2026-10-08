property node, nodelists, nodekeys, chairs, view_mgr, dividers, empty_tables, full_tables, length, room_mgr, letters, shapeinfo, tiles, destTiles, type, barmems

on new me, Args
  type = #bar
  room_mgr = Args.room_mgr
  node = room_mgr.game_mgr.node.newNode(getNodeData("bar"))
  length = 8
  empty_tables = []
  full_tables = []
  letters = ["A", "B", "C", "D", "E", "F", "G", "H"]
  letters.sort()
  tiles = []
  destTiles = []
  barmems = [:]
  repeat with i in ["diner", "family", "finedining", "chezflos"]
    barmems[i] = member("bar", "gfx.diner")
  end repeat
  barmems["goddess"] = member("bar", "gfx.goddess")
  empty_views = []
  full_views = []
  chairs = []
  repeat with i = 1 to length
    dat = Args.chairs[i]
    chairs.add(script("chair", "scripts.game").new([#table: VOID, #data: dat]))
  end repeat
  dividers = []
  repeat with i = 1 to length - 1
    dividers.add(node["divider_" & i])
  end repeat
  nodekeys = [#sitter, #legs, #stool, #stoolnum, #tablestuff, #mousetrap, #dirty, #setting]
  nodelists = [:]
  repeat with i in nodekeys
    nodelists[i] = []
  end repeat
  repeat with i = 1 to length
    repeat with k in nodekeys
      nodelists[k].add(node[k & "_" & i])
    end repeat
  end repeat
  shapeinfo = []
  repeat with i in chairs
    shapeinfo.add([i, me, me])
  end repeat
  shapeinfo.deleteAt(8)
  return me
end

on init me, dt
  destTiles = dt
  me.align()
  xPos = chairs[1].Tile.pos[1] + 2
  me.makeEmptyTable(1, length)
end

on setTile me, t
  tiles.add(t)
end

on setRestaurant me, r
  mem = barmems[r]
  if voidp(mem) then
    return 
  end if
  node["bg"].setMember(mem)
end

on zsortSitters me
  ls = [:]
  ls.sort()
  nodes = nodelists.sitter
  repeat with n in nodes
    ls.addProp(n.y, n)
  end repeat
  repeat with n in ls
    n.moveToFront()
  end repeat
end

on align me
  t = chairs[1].Tile
  if voidp(t) then
    node.setVisible(0)
    return 
  end if
  anchor = t.node
  if voidp(anchor) then
    return 
  end if
  node.setRegPoint(nodelists.stool[1].getLoc() + point(-10, -20))
  node.setParent(anchor)
  node.moveToBack()
  node.setLoc(anchor.getRegPoint())
  node.setVisible(1)
end

on makeEmptyTable me, start_indx, end_indx
  repeat with t in empty_tables
    if (t.startIndex <= end_indx) and (t.endIndex >= start_indx) then
      complain("tried to make overlapping table")
      return 
    end if
  end repeat
  nds = nodelists.duplicate()
  nds = [:]
  chs = []
  repeat with k in nodekeys
    nds[k] = []
  end repeat
  repeat with i = start_indx to end_indx
    chs.add(chairs[i])
    repeat with k in nodekeys
      nds[k].add(nodelists[k][i])
    end repeat
  end repeat
  t = script("table").new([#nodelists: nds, #bar: me, #chairs: chs, #startIndex: start_indx, #room_mgr: room_mgr, #tablenum: 100 + start_indx, #bar_mgr: me, #endIndex: end_indx])
  v = script("bar_view").new(nds, node["mousearea"], me)
  v.setObject(t)
  v.setEmpty()
  t.addView(v)
  repeat with i = start_indx to end_indx - 1
    dividers[i].setVisible(0)
  end repeat
  if end_indx < length then
    dividers[end_indx].setVisible(1)
  end if
  empty_tables.add(t)
  tindx = t.startIndex + integer(float(t.endIndex - t.startIndex) / 2)
  Tile = tiles[tindx]
  t.setTile(Tile)
  t.setDestTile(destTiles[tindx])
  Tile.setOccupant(t)
  return t
end

on removeEmptyTable me, t
  return empty_tables.deleteOne(t)
end

on makeFullTable me, start_indx, end_indx
  container = VOID
  repeat with et in empty_tables
    if (et.startIndex <= start_indx) and (et.endIndex >= end_indx) then
      container = et
      exit repeat
    end if
  end repeat
  if voidp(container) then
    return 
  end if
  contStart = container.startIndex
  contEnd = container.endIndex
  me.removeEmptyTable(container)
  container.destroy()
  if contStart < start_indx then
    me.makeEmptyTable(contStart, start_indx - 1)
  end if
  if end_indx < contEnd then
    me.makeEmptyTable(end_indx + 1, contEnd)
  end if
  nds = [:]
  repeat with k in nodekeys
    nds[k] = []
  end repeat
  chs = []
  repeat with i = start_indx to end_indx
    repeat with k in nodekeys
      nds[k].add(nodelists[k][i])
    end repeat
    chs.add(chairs[i])
  end repeat
  repeat with i = start_indx to end_indx - 1
    dividers[i].setVisible(0)
  end repeat
  if end_indx < length then
    dividers[end_indx].setVisible(1)
  end if
  t = script("table").new([#nodelists: nds, #bar: me, #chairs: chs, #startIndex: start_indx, #room_mgr: room_mgr, #tablenum: 100 + start_indx, #bar_mgr: me, #endIndex: end_indx])
  v = script("bar_view").new(nds, node["mousearea"], me)
  v.setObject(t)
  t.addView(v)
  if full_tables.getOne(t) = 0 then
    full_tables.add(t)
  else
    nothing()
  end if
  tindx = t.startIndex + integer(float(t.endIndex - t.startIndex) / 2)
  Tile = tiles[tindx]
  t.setTile(Tile)
  t.setDestTile(destTiles[tindx])
  Tile.setOccupant(t)
  return t
end

on getLetter me
  let = letters[1]
  letters.deleteAt(1)
  return let
end

on returnLetter me, let
  if (letters.getOne(let) = 0) and stringp(let) then
    letters.add(let)
  end if
end

on removeFullTable me, t
  if not full_tables.deleteOne(t) then
    return 
  end if
  newstart = t.startIndex
  newend = t.endIndex
  if t.startIndex > 1 then
    repeat with et in empty_tables
      if et.endIndex = (newstart - 1) then
        newstart = et.startIndex
        me.removeEmptyTable(et)
        et.destroy()
        exit repeat
      end if
    end repeat
  end if
  if t.endIndex < length then
    repeat with i = 1 to empty_tables.count
      et = empty_tables[i]
      if et.startIndex = (newend + 1) then
        newend = et.endIndex
        me.removeEmptyTable(et)
        et.destroy()
      end if
    end repeat
  end if
  if (newstart = t.startIndex) and (newend = t.endIndex) then
    nothing()
  end if
  t.destroy()
  t = me.makeEmptyTable(newstart, newend)
  if not t then
    nothing()
  end if
  return 1
end

on destroy me
  call(#destroy, empty_tables)
  call(#destroy, full_tables)
  call(#destroy, chairs)
  node.destroy()
  repeat with i = 1 to me.count
    me[i] = VOID
  end repeat
end
