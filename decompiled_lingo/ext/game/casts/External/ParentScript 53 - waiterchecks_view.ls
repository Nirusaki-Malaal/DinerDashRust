property node, num, checknodes, bonus

on new me, n
  node = n
  num = node["num"]
  checknodes = [num, node["check"], node["bg"]]
  call(#setVisible, checknodes, 0)
  node.setVisible(1)
  bonus = script("bonus_flash").new(node["bonus"])
  return me
end

on destroy me
  bonus.destroy()
  bonus = VOID
  checknodes.deleteAll()
  num = VOID
  node = VOID
end

on setChain me, numchecks
  if numchecks < 2 then
    call(#setVisible, checknodes, 0)
  else
    num.setText("x" & string(numchecks))
    call(#setVisible, checknodes, 1)
  end if
end

on showBonus me, bnum
  bonus.show("+" & bnum)
end

on hide me
  node.setVisible(0)
end
