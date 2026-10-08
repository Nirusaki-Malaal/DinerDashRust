global glob

on new me, Args
  if voidp(Args) then
    Args = [:]
  end if
  dat = getNodeData("6top")
  node = glob.managers.game.room.node.newNode(dat)
  Args[#node] = node
  node.setLoc(point(-5000, -5000))
  Args[#capacity] = 6
  Args[#type] = "6top"
  Args[#destroyNode] = 1
  t = script("table_view").new(Args)
  return t
end
