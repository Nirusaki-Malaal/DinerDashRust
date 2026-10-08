on findPath grid, a, b, options
  if voidp(options) then
    options = [:]
  end if
  biaspenalty = 1
  qopen = [:]
  qopenSort = []
  qclosed = [:]
  giveupTime = the milliSeconds + 1000
  sort(qopenSort)
  startNode = [#totalCost: 0, #costFromStart: 0, #costToGoal: pathCostEstimate(a, b), #pos: a, #parent: VOID]
  startNode.totalCost = startNode.costToGoal
  qopen.addProp(startNode.pos, startNode)
  qopenSort.add(startNode)
  repeat while qopen.count > 0
    node = qopenSort[1]
    qopenSort.deleteAt(1)
    qopen.deleteProp(node.pos)
    if node.pos = b then
      return constructPath(node)
    end if
    repeat with newTile in grid.getNeighbors(node.pos, #empty)
      newNodePos = newTile.pos
      traversecost = 1
      delta = newNodePos - node.pos
      if delta[1] and (options[#bias] = #vertical) then
        traversecost = traversecost + biaspenalty
      end if
      if delta[2] and (options[#bias] = #horizontal) then
        traversecost = traversecost + biaspenalty
      end if
      newCostFromStart = node.costFromStart + traversecost
      openNode = qopen[newNodePos]
      closedNode = qclosed[newNodePos]
      if not voidp(openNode) then
        if (newCostFromStart < openNode.costFromStart) or ((newCostFromStart = openNode.costFromStart) and (random(2) = 1)) then
          openNode.costFromStart = newCostFromStart
          openNode.totalCost = openNode.costFromStart + openNode.costToGoal
          openNode.parent = node
        end if
        next repeat
      end if
      if not voidp(closedNode) then
        if newCostFromStart < closedNode.costFromStart then
          qclosed.deleteProp(newNodePos)
          qopen.addProp(newNodePos, closedNode)
          qopenSort.add(closedNode)
          closedNode.costFromStart = newCostFromStart
          closedNode.totalCost = closedNode.costFromStart + closedNode.costToGoal
          closedNode.parent = node
        end if
        next repeat
      end if
      newNode = [#totalCost: 0, #costFromStart: newCostFromStart, #costToGoal: pathCostEstimate(newNodePos, b), #pos: newNodePos, #parent: node]
      newNode.totalCost = newNode.costFromStart + newNode.costToGoal
      qopen.addProp(newNode.pos, newNode)
      qopenSort.add(newNode)
    end repeat
    biaspenalty = biaspenalty * 0.90000000000000002
    qclosed.addProp(node.pos, node)
    if the milliSeconds > giveupTime then
      closest = 1000000
      closestNode = VOID
      repeat with n in qopen
        if n.costToGoal < closest then
          closest = n.costToGoal
          closestNode = n
        end if
      end repeat
      repeat with n in qclosed
        if n.costToGoal < closest then
          closest = n.costToGoal
          closestNode = n
        end if
      end repeat
      put "mapdisplay.findpath: Timed out!"
      if voidp(closestNode) then
        return VOID
      end if
      return constructPath(closestNode)
    end if
  end repeat
  return VOID
end

on pathCostEstimate a, b
  overestimate = 1.5
  return overestimate * manhattan(a, b)
end

on constructPath node
  path = [node.pos]
  repeat while not voidp(node.parent)
    path.addAt(1, node.parent.pos)
    node = node.parent
  end repeat
  return path
end

on manhattan a, b
  return abs(a[1] - b[1]) + abs(a[2] - b[2])
end

on testPath
  n = script("room").new()
  n.loadLevel([#width: 10, #height: 10])
  repeat with xxx = 1 to 3
    repeat with yyy = 1 to 15
      n.grid[random(10)][random(10)].occupant = [#type: #Full]
    end repeat
    p1 = point(random(10), random(10))
    p2 = point(random(10), random(10))
    n.getTileAt(p1).setOccupant(VOID)
    n.getTileAt(p2).setOccupant(VOID)
    pth = findPath(n, p1, p2)
    if voidp(pth) then
      next repeat
    end if
    repeat with row = 1 to n.gridsize[2]
      str = EMPTY
      repeat with col = 1 to n.gridsize[1]
        i = n.grid[row][col]
        if pth.getOne(point(col, row)) then
          ch = "x"
        else
          ch = string(i.occupant.type <> #empty)
        end if
        str = str & ch
      end repeat
      put str
    end repeat
    put EMPTY
  end repeat
  n.destroy()
end
