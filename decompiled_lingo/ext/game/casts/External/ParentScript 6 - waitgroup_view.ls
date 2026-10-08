property sitterGroup, node, mousehandler, dragicon, dragging, clickedIndex, firstinline, sitterviews, sitternodes, heartrow, queue
global glob

on new me, Args
  node = Args.node
  dragicon = Args.dragicon
  queue = Args.queue
  colormems = [:]
  repeat with i in [#red, #green, #blue, #purple]
    colormems[i] = member(string(i) & ".standing", "gfx.people")
  end repeat
  sitterviews = []
  sitternodes = []
  repeat with i = 1 to 6
    n = node["body_" & i]
    f = node["face_" & i]
    sitterviews.add(script("stander_view", "scripts.game").new([#node: n, #facenode: f, #waitgroup_view: me, #number: i]))
    sitternodes.add(n)
  end repeat
  heartrow = script("heartrow_view").new(node["heartrow"])
  me.setGroup(VOID)
  return me
end

on setFirst me, f
  firstinline = f
  call(#setMouseListening, sitterviews, f)
end

on destroy me
  sitterGroup = VOID
  call(#destroy, sitterviews)
  sitterviews.deleteAll()
  heartrow.destroy()
  heartrow = VOID
  node.setVisible(0)
  node = VOID
  dragicon = VOID
end

on clickCustomer me, indx
  if firstinline and not dragging then
    setcursor(#hand_closed)
    clickedIndex = indx
    dragicon.setGroup(sitterGroup, indx, me)
    node.setVisible(0)
    me.setDragging(1)
  end if
end

on mouseRelease me
  queue.mouseRelease()
end

on update me
  if voidp(sitterGroup) then
    return 
  end if
  heartrow.setHearts(sitterGroup.mood)
end

on startFlashing me, ms
  call(#startFlashing, sitterviews, ms)
end

on setDragging me, d
  if dragging = d then
    return 
  end if
  if d then
    subscribe(me, #mouseRelease)
  else
    if node.parent.mouseWithin() then
      setcursor(#hand_open)
    else
      setcursor(#arrow)
    end if
    unsubscribe(me, #mouseRelease)
    node.setVisible(1)
  end if
  dragging = d
end

on removeObject me, obj
  if sitterGroup = obj then
    me.setGroup(VOID)
  end if
end

on showPenalty me, Args
  glob.managers.view.showPenalty(node, Args)
end

on setGroup me, sg
  if sg = sitterGroup then
    return 
  end if
  me.setDragging(0)
  if not voidp(sitterGroup) then
    sitterGroup.removeView(me)
  end if
  sitterGroup = sg
  if voidp(sitterGroup) then
    node.setVisible(0)
    call(#setSitter, sitterviews, VOID)
    return 
  end if
  sitterGroup.addView(me)
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
end
