on findMember mem, node
  if node.type <> #Branch then
    if node.getMember() = mem then
      return node
    end if
  else
    repeat with i in node.children
      n = findMember(mem, i)
      if not voidp(n) then
        return n
      end if
    end repeat
  end if
end

on findSprite mem
  repeat with i = 1 to the lastChannel
    if sprite(i).member = mem then
      return i
    end if
  end repeat
  put "no sprite found with member" && mem
end

on findLeaf spr, node
  if node.type = #leaf then
    if node.pSprite.spriteNum = spr then
      return node
    end if
  else
    repeat with i in node.children
      n = findLeaf(spr, i)
      if not voidp(n) then
        return n
      end if
    end repeat
  end if
end

on setSpriteVis vis, rng
  if voidp(rng) then
    rng = [1, the lastChannel]
  end if
  repeat with i = rng[1] to rng[2]
    sprite(i).visible = vis
  end repeat
end
