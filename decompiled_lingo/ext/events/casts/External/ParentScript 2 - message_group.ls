property enabled, listeners, handler, Args, dirty

on new me, hndlr, rgs, startDisabled
  handler = hndlr
  Args = rgs
  listeners = []
  enabled = not startDisabled
  return me
end

on destroy me
  listeners.deleteAll()
  Args = VOID
end

on subscribe me, obj
  if obj.handlers().getOne(handler) = 0 then
    complain(string(obj) & " has no " & string(handler) & " handler")
    return 
  end if
  if listeners.getOne(obj) = 0 then
    listeners.add(obj)
  end if
end

on unsubscribe me, obj
  indx = listeners.getOne(obj)
  if indx = 0 then
    return 
  end if
  listeners[indx] = VOID
  dirty = 1
end

on clean me
  repeat while listeners.getOne(VOID)
    listeners.deleteOne(VOID)
  end repeat
  dirty = 0
end

on send me, pass_args
  if enabled then
    call(handler, listeners, pass_args, Args)
  end if
  if dirty then
    me.clean()
  end if
end

on setEnabled me, e
  enabled = e
end
