property drag_mgr, callbacks, cb_keys, node, currentTarget, group

on new me, Args
  group = Args.targetGroup
  callbacks = [:]
  cb_keys = [#dragEnter, #dragLeave, #dragDrop, #dragRelease]
  return me
end

on setCallback me, sym, cb
  if cb_keys.getOne(sym) = 0 then
    return 
  end if
  callbacks[sym] = cb
end

on clearCallbacks me
  callbacks.deleteAll()
end

on forwardEvents me, obj
  me.clearCallbacks()
  h = obj.handlers()
  repeat with i in cb_keys
    if h.getOne(i) <> 0 then
      me.setCallback(i, [#target: obj, #handler: i])
    end if
  end repeat
end

on destroy me
  data = VOID
  callbacks.deleteAll()
  obj = VOID
  group.removeSource(me)
  group = VOID
  currentTarget = VOID
end

on startDragging me
  group.setSource(me)
end

on stopDragging me
  group.removeSource(me)
end

on release me
  if voidp(currentTarget) then
    me.dragRelease()
  else
    me.dragDrop(currentTarget)
  end if
end

on dragEnter me, t
  if t = currentTarget then
    return 
  end if
  currentTarget = t
  cb = callbacks[#dragEnter]
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb[#target], t.data, cb[#Args])
end

on dragLeave me, t
  if t = currentTarget then
    currentTarget = VOID
  end if
  cb = callbacks[#dragLeave]
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb[#target], t.data, cb[#Args])
end

on dragRelease me
  cb = callbacks[#dragRelease]
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb[#target], cb[#Args])
end

on dragDrop me, t
  cb = callbacks[#dragDrop]
  if voidp(cb) then
    return 
  end if
  call(cb.handler, cb[#target], t.data, cb[#Args])
end
