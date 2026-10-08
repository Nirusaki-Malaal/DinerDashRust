property Tile, node, shapeinfo, type

on new me, Args
  type = #obstacle
  node = Args.parentNode.newNode(getNodeData("obstacle").children[1])
  shapeinfo = [[me]]
  return me
end

on destroy me
  if not voidp(Tile) then
    Tile.removeOccupant(me)
  end if
  Tile = VOID
  node.destroy()
  node = VOID
end

on setTile me, t
  Tile = t
  node.setParent(Tile.node)
  node.setLoc(Tile.node.getRegPoint())
end

on getTile me
  return Tile
end
