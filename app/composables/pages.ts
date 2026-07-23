export const usePages = () => {
	const router = useRouter();
	const { pageCategories } = useAppConfig();
	const { user } = useAuth();

	const routes = router.getRoutes().filter(
		(route) => route.name !== "index" && route.name !== "all"
	);

	const categorizedRoutes = routes.reduce((acc, route) => {
		const category = route.meta.category as string || "other";
		if (!category) return acc;

		// Role-based route filtering (story 1.5)
		// If a page defines minRole, only users with that role (or admin) can see it
		const minRole = route.meta.minRole as string | undefined;
		if (minRole) {
			const userRole = user.value?.role;
			// No user logged in → hide restricted pages
			if (!userRole) return acc;
			// Only admin or matching role can see the page
			if (userRole !== "admin" && userRole !== minRole) {
				return acc;
			}
		}

		if (!acc[category]) {
			acc[category] = {
				label: pageCategories[category as keyof typeof pageCategories]?.label,
				icon: pageCategories[category as keyof typeof pageCategories]?.icon || "i-lucide-folder",
				to: route.path,
				children: []
			};
		}

		acc[category].children.push({
			label: route.meta.name as string || route.name,
			description: route.meta.description as string,
			icon: route.meta.icon || "i-lucide-file",
			to: route.path
		});

		return acc;
	}, {} as Record<string, any>);

	const pages = Object.values(categorizedRoutes);

	return {
		pages
	};
};
