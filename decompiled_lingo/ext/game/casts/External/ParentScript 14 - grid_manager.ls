property room_mgr, grid, size, view, podiumPath

on new me, Args
  room_mgr = Args.room_manager
  size = [0, 0]
  grid = []
  view = script("grid_view").new([#grid_manager: me, #parentNode: room_mgr.node])
  return me
end

on destroy me
  if not voidp(view) then
    view.destroy()
    view = VOID
  end if
  me.destroyGrid()
end

on blockColumn me
  repeat with i = 1 to min(grid.count, 9)
    grid[i][1].setBlocked()
    grid[i][2].setBlocked()
  end repeat
end

on destroyGrid me
  size = [0, 0]
  repeat with i in grid
    call(#destroy, i)
    i.deleteAll()
  end repeat
  grid.deleteAll()
end

on getTileNearest me, pt
  return me.getTileAt(view.getPosNearest(pt))
end

on setSize me, sz
  me.destroyGrid()
  last_row = VOID
  rowB4last = VOID
  repeat with i = 1 to sz[2]
    row = []
    last_tile = VOID
    tileB4last = VOID
    repeat with j = 1 to sz[1]
      t = script("tile", "scripts.game").new([#manager: me, #pos: point(j, i)])
      if not voidp(last_tile) then
        t.addNeighbor(last_tile)
        last_tile.addNeighbor(t)
      end if
      if not voidp(last_row) then
        above = last_row[j]
        t.addNeighbor(above)
        above.addNeighbor(t)
      end if
      if not voidp(tileB4last) then
        t.addDoubleNeighbor(tileB4last)
        tileB4last.addDoubleNeighbor(t)
      end if
      if not voidp(rowB4last) then
        above2 = rowB4last[j]
        t.addDoubleNeighbor(above2)
        above2.addDoubleNeighbor(t)
      end if
      row.add(t)
      tileB4last = last_tile
      last_tile = t
    end repeat
    if i < 3 then
      call(#setBlocked, row)
    end if
    grid.add(row)
    rowB4last = last_row
    last_row = row
  end repeat
  size = sz
  view.update()
end

on getTileAt me, x, y
  if x.listp then
    y = x[2]
    x = x[1]
  end if
  if (x < 1) or (x > size[1]) or (y < 1) or (y > size[2]) then
    return VOID
  end if
  return grid[y][x]
end

on getNeighbors me, pos, tp
  Tile = me.getTileAt(pos)
  ls = Tile.getNeighbors(tp)
  return Tile.getNeighbors(tp)
end

on putOccupants
  repeat with i in grid
    str = EMPTY
    repeat with j in i
      str = str & j.occupant.type & SPACE
    end repeat
    put str
  end repeat
end

on caniplace me, obj, pos
  si = obj[#shapeinfo]
  if voidp(si) then
    si = [[obj]]
  end if
  y = pos[2]
  repeat with row in si
    x = pos[1]
    repeat with obj in row
      if not voidp(obj) then
        Tile = me.getTileAt(x, y)
        if voidp(Tile) then
          if obj = #spacer then
            next repeat
          else
            return 0
          end if
        end if
        tp = Tile.occupant.type
        if (tp <> #empty) and (tp <> #Full) then
          return 0
        end if
      end if
      x = x + 1
    end repeat
    y = y + 1
  end repeat
  return 1
end

on updateLocZ me
  view.node.updateLocZ()
end

on placeObject me, obj, pos
  pos = pos + [1, 0]
  if not me.caniplace(obj, pos) then
    return 0
  end if
  si = obj[#shapeinfo]
  if voidp(si) then
    si = [[obj]]
  end if
  y = pos[2]
  repeat with row in si
    gridrow = grid[y]
    x = pos[1]
    repeat with obj in row
      if obj = #block then
        Tile = gridrow[x]
        Tile.setBlocked()
      else
        if not voidp(obj) and (obj <> #spacer) then
          Tile = gridrow[x]
          Tile.setOccupant(obj)
          obj.setTile(Tile)
        end if
      end if
      x = x + 1
    end repeat
    y = y + 1
  end repeat
  return 1
end
