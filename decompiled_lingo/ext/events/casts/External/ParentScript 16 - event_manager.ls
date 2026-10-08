property routers, voidRouter, paused

on new me
  voidRouter = script("message_router").new()
  routers = [:]
  pausedGroups = []
  addedGroups = []
  deletedGroups = []
  return me
end

on init me
end

on destroy me
  voidRouter.destroy()
  call(#destroy, routers)
  repeat with i = 1 to me.count
    me[i] = VOID
  end repeat
end

on subscribe me, obj, evt, grp
  if voidp(grp) then
    r = voidRouter
  else
    r = routers[grp]
    if voidp(r) then
      r = script("message_router").new()
      routers[grp] = r
    end if
  end if
  r.subscribe(obj, evt)
end

on setGroupPaused me, p, grp
  if voidp(grp) then
    voidRouter.setPaused(p)
    return 
  end if
  g = routers[grp]
  if voidp(g) then
    return 
  end if
  g.setPaused(p)
end

on setPaused me, p
  paused = p
end

on unsubscribe me, obj, evt, grp
  if voidp(grp) then
    voidRouter.unsubscribe(obj, evt)
  else
    g = routers[grp]
    if not voidp(g) then
      g.unsubscribe(obj, evt)
    end if
  end if
end

on sendEvent me, evt, dat, grp
  if paused then
    return 
  end if
  if voidp(grp) then
    voidRouter.sendEvent(evt, dat)
    call(#sendEvent, routers, evt, dat)
  else
    routers[grp].sendEvent(evt, dat)
  end if
end
