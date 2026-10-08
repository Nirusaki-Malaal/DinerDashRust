property Tile, node, type, shapeinfo

on new me, Args
  type = Args.type
  node = Args.node
  shapeinfo = [[me]]
  return me
end

on setTile me, t
  Tile = t
  if voidp(Tile.node) then
    return 
  end if
  node.setParent(Tile.node)
end

on destroy me
  node = VOID
  if not voidp(Tile) then
    Tile.removeOccupant(me)
    Tile = VOID
  end if
end
