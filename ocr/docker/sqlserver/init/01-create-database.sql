IF NOT EXISTS (SELECT name FROM sys.databases WHERE name = N'kie_pla')
BEGIN
    CREATE DATABASE [kie_pla];
END
GO
