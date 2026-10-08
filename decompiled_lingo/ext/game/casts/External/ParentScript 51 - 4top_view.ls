global glob

on new me, Args
  if voidp(Args) then
    Args = [:]
  end if
  dat = getNodeData("4top")
  node = glob.managers.game.room.node.newNode(dat)
  Args[#node] = node
  node.setLoc(point(-5000, -5000))
  Args[#capacity] = 4
  Args[#type] = "4top"
  Args[#destroyNode] = 1
  return script("table_view").new(Args)
end
