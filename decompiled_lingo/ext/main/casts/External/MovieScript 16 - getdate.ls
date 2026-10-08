on daysUntil m, d, y
  dstring = date()
  i = the itemDelimiter
  the itemDelimiter = "/"
  if dstring.item.count <> 3 then
    return -1
  end if
  today = date(integer(dstring.item[3]), integer(dstring.item[1]), integer(dstring.item[2]))
  the itemDelimiter = i
  return date(y, m, d) - today
end
