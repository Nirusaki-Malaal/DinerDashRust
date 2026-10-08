property Tile, node
global glob

on new me, Args
  Tile = Args.Tile
  node = Args.node
  Tile.node = node
  return me
end

on setTile me, t
  if Tile = t then
    return 
  end if
  Tile = t
  if voidp(Tile) then
    node.setVisible(0)
  end if
end

on destroy me
  Tile = VOID
  if voidp(node) then
    return 
  end if
  node.destroy()
  node = VOID
end
