property node, room_mgr, game_mgr, dragicon, wgnodes, size

on new me, Args
  size = 8
  room_mgr = Args.room_manager
  game_mgr = Args.game_manager
  node = Args[#node]
  grp = getNodeGroup("dragndrop")
  dragnode = grp.newNode(getNodeData("waitgroup"))
  dragnode.setRegPoint(dragnode.children[1].getLoc())
  dragnode.moveToFront()
  dragicon = script("waitgroup_drag").new([#node: dragnode, #queue: me])
  dragicon.setVisible(0)
  wgnodes = []
  repeat with i = 1 to size
    n = node["group_" & string(i)]
    if voidp(n) then
      complain("missing waitgroup")
      next repeat
    end if
    wgnodes.add(script("waitgroup_view").new([#node: n, #dragicon: dragicon, #queue: me]))
  end repeat
  return me
end

on destroy me
  room_mgr = VOID
  game_mgr = VOID
  node = VOID
  call(#destroy, wgnodes)
  wgnodes.deleteAll()
  dragicon.destroy()
  dragicon = VOID
end

on activatePodium me
  wgroups = game_mgr.waitingGroups
  ct = wgroups.count
  ms = 0
  repeat with i = 1 to wgnodes.count
    if i > ct then
      return 
    end if
    n = wgnodes[i]
    n.startFlashing(ms)
    ms = ms + 200
  end repeat
end

on mouseRelease me
  call(#setDragging, wgnodes, 0)
end

on update me
  wgroups = game_mgr.waitingGroups
  ct = wgroups.count
  if ct = 0 then
    call(#setGroup, wgnodes, VOID)
    return 
  end if
  repeat with i = 1 to wgnodes.count
    n = wgnodes[i]
    n.setFirst(1)
    if i > ct then
      n.setGroup(VOID)
      next repeat
    end if
    n.setGroup(wgroups[i])
  end repeat
end
