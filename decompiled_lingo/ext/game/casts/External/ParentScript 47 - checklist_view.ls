property waiter, checks, names

on new me, Args
  waiter = Args.waiter
  checks = []
  n = Args.room.node["checklist"]
  repeat with i = 1 to 8
    checks.add(script("action_view").new(n["action_" & i]))
  end repeat
  me.updateChain()
  return me
end

on updateAll me
  me.updateChain()
end

on updateChain me
  q = waiter.checkchain
  ct = q.count
  repeat with i = 1 to 8
    if i <= ct then
      checks[i].showDest(q[i], #check)
      next repeat
    end if
    checks[i].setVisible(0)
  end repeat
end

on destroy me
  call(#destroy, checks)
  checks.deleteAll()
  waiter = VOID
end
