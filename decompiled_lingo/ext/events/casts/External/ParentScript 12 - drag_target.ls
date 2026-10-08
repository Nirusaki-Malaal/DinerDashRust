property mousehandler, source, groups, data

on new me, Args
  mousehandler = newMouseListener()
  mousehandler.addNode(Args.node, Args[#recursive])
  mousehandler.forwardEvents(me)
  data = Args[#data]
  groups = []
  return me
end

on addNode me, n
  mousehandler.addNode(n)
end

on addGroup me, g
  if groups.getOne(g) = 0 then
    groups.add(g)
  end if
end

on removeGroup me, g
  groups.deleteOne(g)
  g.removeTarget(me)
end

on removeAllGroups me, g
  call(#removeTarget, groups, me)
  groups.deleteAll()
end

on setData me, d
  if data = d then
    return 
  end if
  if not voidp(data) then
    data = VOID
  end if
  data = d
end

on setSource me, d
  if d = source then
    return 
  end if
  if not voidp(source) then
    source.dragLeave(me, data)
    source = VOID
  end if
  source = d
  if voidp(source) then
    mousehandler.clearCallbacks()
    return 
  else
    mousehandler.forwardEvents(me)
  end if
end

on destroy me
  me.removeAllGroups()
  mousehandler.destroy()
  mousehandler = VOID
  obj = VOID
end

on mouseEnter me
  if voidp(source) then
    return 
  end if
  source.dragEnter(me, data)
end

on mouseLeave me
  if voidp(source) then
    return 
  end if
  source.dragLeave(me, data)
end
