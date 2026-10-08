property targets, dragsource, mousepressed

on new me
  targets = []
  return me
end

on destroy me
  call(#removeGroup, targets, me)
  targets.deleteAll()
end

on addTarget me, t
  if targets.getOne(t) <> 0 then
    return 
  end if
  targets.add(t)
  t.addGroup(me)
end

on removeTarget me, t
  if targets.getOne(t) = 0 then
    return 
  end if
  t.setSource(VOID)
  targets.deleteOne(t)
  t.removeGroup(me)
end

on moveObject me, srcobj, tobj
end

on setSource me, dobj
  if dragsource = dobj then
    return 
  end if
  if not voidp(dragsource) then
    dragsource = VOID
  end if
  dragsource = dobj
  call(#setSource, targets, dobj)
end

on removeSource me, s
  if dragsource = s then
    me.setSource(VOID)
  end if
end
