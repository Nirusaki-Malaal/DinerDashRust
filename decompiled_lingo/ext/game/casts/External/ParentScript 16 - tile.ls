property occupant, emptyOccupant, fullOccupant, pos, neighbors, doubleNeighbors, grid_mgr, node

on new me, Args
  pos = Args.pos
  emptyOccupant = [#type: #empty]
  fullOccupant = [#type: #Full]
  occupant = emptyOccupant
  neighbors = []
  doubleNeighbors = []
  return me
end

on setOccupant me, obj
  occupant = obj
  if voidp(occupant) then
    occupant = emptyOccupant
  end if
end

on removeOccupant me, obj
  if occupant = obj then
    occupant = emptyOccupant
  end if
end

on addNeighbor me, obj
  if neighbors.getOne(obj) = 0 then
    neighbors.add(obj)
  end if
end

on addDoubleNeighbor me, obj
  if doubleNeighbors.getOne(obj) = 0 then
    doubleNeighbors.add(obj)
  end if
end

on getNeighbors me, tp
  if voidp(tp) then
    return neighbors
  end if
  if not tp.listp then
    tp = [tp]
  end if
  ls = []
  repeat with i in neighbors
    if tp.getOne(i.occupant.type) then
      ls.add(i)
    end if
  end repeat
  return ls
end

on getDoubleNeighbors me, tp
  if voidp(tp) then
    return doubleNeighbors
  end if
  if not tp.listp then
    tp = [tp]
  end if
  ls = []
  repeat with i in doubleNeighbors
    if tp.getOne(i.occupant.type) then
      ls.add(i)
    end if
  end repeat
  return ls
end

on setBlocked me
  occupant = fullOccupant
end

on getNeighborOccupants me, tp
  ls = []
  repeat with i in neighbors
    if voidp(tp) or (i.occupant.type = tp) then
      ls.add(i.occupant)
    end if
  end repeat
  return ls
end

on getDoubleNeighborOccupants me, tp
  ls = []
  repeat with i in doubleNeighbors
    if voidp(tp) or (i.occupant.type = tp) then
      ls.add(i.occupant)
    end if
  end repeat
  return ls
end

on getDistanceTo me, n
  return abs(pos[1] - n.pos[1]) + abs(pos[2] - n.pos[2])
end

on destroy me
  grid_mgr = VOID
  neighbors.deleteAll()
  doubleNeighbors.deleteAll()
  occupant = VOID
end

on getAt me, indx
  if indx = 1 then
    return pos[1]
  end if
  if indx = 2 then
    return pos[2]
  end if
end

on setNode me, n
  node = n
end
