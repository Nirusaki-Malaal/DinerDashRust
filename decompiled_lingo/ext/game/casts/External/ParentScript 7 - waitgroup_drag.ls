property sitterGroup, node, sitterviews, standinglocs, dragging, dragsource, heartrow, table, tableNode, clickedIndex, rolledChair, queue
global glob

on new me, Args
  node = Args.node
  node.setZOffset(1000000)
  queue = Args.queue
  sitterviews = []
  standinglocs = []
  repeat with i = 1 to 6
    n = node["body_" & i]
    f = node["face_" & i]
    standinglocs.add(n.getLoc())
    sitterviews.add(script("stander_view", "scripts.game").new([#node: n, #facenode: f, #drag: 1]))
  end repeat
  dragGroup = getDragGroup("game.tables")
  dragsource = script("drag_source").new([#targetGroup: dragGroup])
  dragsource.forwardEvents(me)
  heartrow = script("heartrow_view").new(node["heartrow"])
  return me
end

on destroy me
  sitterGroup = VOID
  call(#destroy, sitterviews)
  sitterviews.deleteAll()
  dragsource.destroy()
  dragsource = VOID
  heartrow.destroy()
  heartrow = VOID
  node.setVisible(0)
  table = VOID
  tableNode = VOID
  rolledChair = VOID
  node = VOID
end

on exitFrame me
  if not (the stillDown) then
    me.dragRelease()
    return 
  end if
  if voidp(table) then
    node.setLoc(the mouseLoc)
  else
    if not voidp(tableNode) then
      mw = tableNode.mouseWithin(1)
      nothing()
    end if
    if not mw then
      if table.type = #bar then
      end if
      table.placeSitters(VOID)
      table = VOID
      tableNode = VOID
      rolledChair = VOID
      me.setGroup(sitterGroup)
      node.setVisible(1)
    end if
  end if
end

on setVisible me, v
  node.setVisible(v)
end

on setDragging me, b
  dragging = b
  if b then
    if not voidp(sitterGroup) then
      sitterGroup.setPickedUp(1)
    end if
    subscribe(me, [#exitFrame])
    dragsource.startDragging()
    node.setVisible(1)
    node.setVisible(1)
    me.exitFrame()
  else
    if not voidp(sitterGroup) then
      sitterGroup.setPickedUp(0)
    end if
    call(#setDragging, queue.wgnodes, 0)
    unsubscribe(me, [#exitFrame])
    dragsource.stopDragging()
    node.setVisible(0)
  end if
end

on update me
  if voidp(sitterGroup) then
    return 
  end if
  m = sitterGroup.mood
  heartrow.setHearts(m)
end

on setGroup me, sg, indx
  if voidp(node) then
    return 
  end if
  if not voidp(sitterGroup) and (sg <> sitterGroup) then
    sitterGroup.removeView(me)
  end if
  sitterGroup = sg
  if voidp(sitterGroup) then
    node.setVisible(0)
    call(#setSitter, sitterviews, VOID)
    me.setDragging(0)
    return 
  end if
  if (indx < 1) or (indx > sitterGroup.sitters.count) then
    indx = 1
  end if
  clickedIndex = indx
  sitterGroup.addView(me)
  node.setRegPoint(standinglocs[indx])
  me.update()
  ss = sitterGroup.sitters
  repeat with i = 1 to 6
    n = sitterviews[i]
    if i > ss.count then
      n.setSitter(VOID)
      next repeat
    end if
    s = ss[i]
    n.setSitter(s)
  end repeat
  node.setVisible(1)
  me.setDragging(1)
end

on removeObject me, sg
  me.setGroup(VOID)
end

on removeGroup me, sg
  me.setGroup(VOID)
end

on dragDrop me, ls
  if voidp(ls) then
    if voidp(table) then
      return 
    end if
    ls = [#table: table, #chair: rolledChair]
  end if
  t = ls.table
  table = VOID
  tableNode = VOID
  rolledChair = VOID
  if voidp(sitterGroup) then
    return 
  end if
  if t.canHold(sitterGroup) then
    sitterGroup.sitAtTable(t, ls[#chair], clickedIndex)
  end if
end

on dragEnter me, ls
  if voidp(ls) then
    return 
  end if
  t = ls.table
  if not voidp(t[#bar]) then
    if table = t then
      nothing()
    end if
  end if
  if not voidp(table) and (t <> table) then
    table.placeSitters(VOID)
  end if
  if t.canHold(sitterGroup) then
    node.setVisible(0)
    table = t
    rolledChair = ls[#chair]
    chair_indx = table.chairs.getOne(ls[#chair])
    if not voidp(t[#bar]) then
      indx = max(1, min(chair_indx - clickedIndex + 1, t.capacity))
    else
      indx = ((chair_indx - clickedIndex + t.capacity) mod t.capacity) + 1
    end if
    table.placeSitters(sitterGroup, indx)
    tableNode = ls.tableNode
  else
    table = VOID
    tableNode = VOID
    node.setVisible(1)
  end if
end

on dragRelease me
  me.setDragging(0)
  if not voidp(table) then
    me.dragDrop([#table: table, #chair: rolledChair])
  end if
end
