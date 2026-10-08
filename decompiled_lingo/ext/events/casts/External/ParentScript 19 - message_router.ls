property groups, dirty, paused

on new me, g
  groups = [:]
  return me
end

on destroy me
  call(#destroy, groups)
  groups.deleteAll()
  glob = VOID
end

on sendEvent me, Msg, Args
  if paused then
    return 
  end if
  gr = groups[Msg]
  if voidp(gr) then
    return 
  end if
  gr.send(Args)
  if gr.listeners.count = 0 then
    gr.destroy()
    groups.deleteProp(Msg)
  end if
end

on subscribe me, obj, Msg
  if not listp(Msg) then
    Msg = [Msg]
  end if
  h = obj.handlers()
  repeat with m in Msg
    if h.getOne(m) = 0 then
      next repeat
    end if
    g = groups[m]
    if voidp(g) then
      g = script("message_group").new(m)
      groups[m] = g
    end if
    g.subscribe(obj)
  end repeat
end

on unsubscribe me, obj, Msg
  if not listp(Msg) then
    Msg = [Msg]
  end if
  h = obj.handlers()
  repeat with m in Msg
    if h.getOne(m) = 0 then
      next repeat
    end if
    g = groups[m]
    if not voidp(g) then
      g.unsubscribe(obj)
      if g.listeners.count = 0 then
        g.destroy()
        groups.deleteProp(m)
      end if
    end if
  end repeat
end

on setPaused me, p
  paused = p
end

on unsubscribeAll me, obj
  call(#unsubscribe, groups, obj)
end
