

## Table: users
|  id  (primary)  |  name  |   public_ssh_key  |
|-----------------|--------|-------------------|
|        1        |  Dz0N  |  ................ |

## Table: flags
|  id (primary)  |  name |   description (md)   |  points  |  flag  |
|----------------|-------|----------------------|----------|--------|
|       1        | flag0 | [an easy flag](link) |  100     |ctf{...}|

## Table: solved
|  id (primary)  |  uid (foreign) |  fid (foreign)  |
|----------------|----------------|-----------------|
|       1        |     1 (Dz0N)   |    1 (flag0)    |

