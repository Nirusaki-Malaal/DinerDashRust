property waiter, actions, names, currentaction

on new me, Args
  waiter = Args.waiter
  actions = []
  n = Args.room.node["actionlist"]
  repeat with i = 2 to 5
    actions.add(script("action_view").new(n["action_" & i]))
  end repeat
  currentaction = script("action_view").new(n["action_1"])
  me.updateQueue()
  return me
end

on updateQueue me
  currentaction.showDest(waiter.currentDest)
  q = waiter.destqueue
  ct = q.count
  repeat with i = 1 to 4
    if i <= ct then
      actions[i].showDest(q[i])
      next repeat
    end if
    actions[i].setVisible(0)
  end repeat
end

on updateAll me
  me.updateQueue()
end

on destroy me
  call(#destroy, actions)
  actions.deleteAll()
  waiter = VOID
end
