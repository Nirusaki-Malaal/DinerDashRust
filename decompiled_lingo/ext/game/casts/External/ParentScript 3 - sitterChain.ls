property elements, count

on new me
  elements = []
  return me
end

on destroy me
  elements.deleteAll()
end

on add me, s
  if elements.getOne(s) = 0 then
    elements.add(s)
    count = elements.count
  end if
end

on Remove me, s
  return elements.deleteOne(s)
end
