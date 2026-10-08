property slots, data

on new me, dat
  data = dat
  slots = []
  return me
end

on connectSlot me, s
  s.addSignal(me)
  me.addSlot(s)
end

on disconnectSlot me, s
  s.removeSignal(me)
  me.removeSlot(s)
end

on disconnectAll me
  call(#removeSignal, slots, me)
  slots.deleteAll()
end

on addSlot me, s
  if slots.getOne(s) = 0 then
    slots.add(s)
  end if
end

on removeSlot me, s
  slots.deleteOne(s)
end

on destroy me
  me.disconnectAll()
  data = VOID
end

on fire me
  call(#fire, slots, data)
end
