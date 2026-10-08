property mgr, num

on new me, m, n
  num = n
  mgr = m
  mgr.subscribe(me, #doit1)
  return me
end

on destroy me
  mgr.unsubscribeAll(me)
end

on test scr
  m = script("message_manager").new()
  ls = []
  repeat with i = 1 to 10
    ls.add(scr.new(m, i))
  end repeat
  m.sendMessage(#doit1)
  m.sendMessage(#doit2)
  m.sendMessage(#doit1)
  call(#destroy, ls)
  ls.deleteAll()
  m.sendMessage(#doit1)
  m.sendMessage(#doit2)
  m.destroy()
  m = VOID
end

on doit1 me
  put "obj" & string(num) & ".doit1()"
  if random(2) = 1 then
    mgr.unsubscribe(me, #doit1)
  end if
  if random(2) = 1 then
    mgr.subscribe(me, #doit2)
  end if
end

on doit2 me
  put "obj" & string(num) & ".doit2()"
end
