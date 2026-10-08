property sg, name, events

on new me, t
  sg = t
  name = #new
  events = []
  return me
end

on destroy me
  sg = VOID
end

on enterState me, previousState
end

on event me, sym
end

on leaveState me, nextState
end
