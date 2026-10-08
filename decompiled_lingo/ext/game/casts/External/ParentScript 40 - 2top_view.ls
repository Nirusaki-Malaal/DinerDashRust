global glob

on new me, Args
  if voidp(Args) then
    Args = [:]
  end if
  node = glob.managers.game.room.node.newNode(getNodeData("2top"))
  node.setLoc(point(-5000, -5000))
  Args[#node] = node
  Args[#capacity] = 2
  Args[#type] = "2top"
  Args[#destroyNode] = 1
  node.updateTransform()
  return script("table_view").new(Args)
end
