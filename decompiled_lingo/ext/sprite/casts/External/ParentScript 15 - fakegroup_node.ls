property children, mouseListeners, type

on new me, nodeList
  type = #Branch
  children = nodeList
  mouseListeners = []
  return me
end

on setVisible me, v
  call(#setVisible, children, v)
end

on setLoc me, pt
  call(#setLoc, children, pt)
end

on mouseWithin me
  repeat with i in children
    if mouseWithin(i) then
      return 1
    end if
  end repeat
end

on destroy me
  children.deleteAll()
  call(#removeNode, mouseListeners, me)
end

on addMouseListener me, x
  if mouseListeners.getOne(x) <> 0 then
    return 
  end if
  call(#addMouseListener, children, x)
end

on removeMouseListener me, x
  call(#removeMouseListener, children, x)
end
