property gridobj, tileviews, node, width, height, tilenodes, podiumPath, tilesize, nodesize

on new me, Args
  gridobj = Args.grid_manager
  node = Args.parentNode["floor"]
  node.setVisible(1)
  node.setAlpha(100)
  width = 18
  height = 13
  tilesize = [37, 37]
  nodesize = tilesize * [width, height]
  tilenodes = []
  repeat with i = 0 to height - 1
    row = []
    repeat with j = 1 to width
      row.add(node.children[(i * width) + j])
    end repeat
    tilenodes.add(row)
  end repeat
  node.setVisible(1)
  ct = node.children.count
  call(#setVisible, node.children, 0)
  tileviews = []
  me.setGrid(gridobj)
  return me
end

on setGrid me, gr
  if gridobj = gr then
    return 
  end if
  gridobj = gr
  call(#destroy, tileviews)
  tileviews.deleteAll()
  if voidp(gridobj) then
    return 
  end if
  g = gridobj.grid
  me.update()
end

on getPosNearest me, pt
  myloc = node.getLoc()
  rel = (pt - myloc + tilesize) / tilesize
  x = max(1, min(rel[1], width))
  y = max(1, min(rel[2], height))
  return [x, y]
end

on destroy me
  call(#destroy, tileviews)
  tileviews.deleteAll()
  grid = VOID
end

on updateLocZ me
  node.updateLocZ()
end

on update me
  if voidp(gridobj) then
    return 
  end if
  g = gridobj.grid
  w = gridobj.size[1]
  h = gridobj.size[2]
  repeat with r = 1 to height
    tr = tilenodes[r]
    if r > h then
      next repeat
    end if
    repeat with c = 1 to width
      n = tr[c]
      if c > w then
        n.setVisible(0)
        next repeat
      end if
      n.setVisible(1)
      Tile = gridobj.grid[r][c]
      Tile.setNode(n)
    end repeat
  end repeat
end
